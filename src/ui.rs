//! Enhanced TUI with real-time charts and sparklines
//!
//! Provides a visually stunning interface with:
//! - Sparklines in the main table view
//! - Expandable detail view with full throughput charts
//! - Color-coded status indicators
//! - Smooth visual transitions

#![allow(clippy::cast_precision_loss)] // Acceptable for chart coordinates
#![allow(clippy::cast_possible_truncation)] // Acceptable for UI values
#![allow(clippy::cast_sign_loss)] // Values are always positive
#![allow(clippy::similar_names)] // rx/tx pairs are intentionally similar

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Cell, Chart, Dataset, GraphType, Paragraph, Row, Table, Tabs},
    Frame,
};

use crate::history::PortHistory;
use crate::metrics::MetricsCollector;
use crate::types::{AdapterInfo, PortState};

/// Number of sparkline samples to show in the main table
const SPARKLINE_SAMPLES: usize = 20;

/// Application state for the UI
#[derive(Debug, Default)]
pub struct AppState {
    /// Currently selected row index (for navigation)
    pub selected_row: usize,
    /// Whether detail view is expanded
    pub detail_expanded: bool,
    /// Currently selected tab in detail view
    pub detail_tab: usize,
    /// Scroll offset for the main table (for future scrolling support)
    #[allow(dead_code)]
    pub scroll_offset: usize,
    /// Animation frame counter
    pub frame_count: u64,
    /// List of selectable items (adapter, port) or None for adapter headers
    selectable_items: Vec<Option<(String, u16)>>,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Move selection up
    pub fn select_prev(&mut self) {
        if self.selected_row > 0 {
            self.selected_row -= 1;
            // Skip adapter header rows
            while self.selected_row > 0 && self.is_header_row(self.selected_row) {
                self.selected_row -= 1;
            }
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected_row + 1 < self.selectable_items.len() {
            self.selected_row += 1;
            // Skip adapter header rows
            while self.selected_row + 1 < self.selectable_items.len()
                && self.is_header_row(self.selected_row)
            {
                self.selected_row += 1;
            }
        }
    }

    /// Check if a row is a header (not selectable)
    fn is_header_row(&self, row: usize) -> bool {
        match self.selectable_items.get(row) {
            None | Some(None) => true,
            Some(Some(_)) => false,
        }
    }

    /// Toggle detail view
    pub fn toggle_detail(&mut self) {
        self.detail_expanded = !self.detail_expanded;
    }

    /// Get currently selected port
    pub fn selected_port(&self) -> Option<(&str, u16)> {
        self.selectable_items
            .get(self.selected_row)?
            .as_ref()
            .map(|(a, p)| (a.as_str(), *p))
    }

    /// Cycle detail tab
    pub fn next_tab(&mut self) {
        self.detail_tab = (self.detail_tab + 1) % 3;
    }

    /// Cycle detail tab backward
    pub fn prev_tab(&mut self) {
        self.detail_tab = if self.detail_tab == 0 {
            2
        } else {
            self.detail_tab - 1
        };
    }

    fn update_selectable_items(&mut self, adapters: &[AdapterInfo]) {
        self.selectable_items.clear();
        for adapter in adapters {
            self.selectable_items.push(None); // Adapter header
            for port in &adapter.ports {
                self.selectable_items
                    .push(Some((adapter.name.clone(), port.port_number)));
            }
        }
        // Ensure selection is valid
        if self.selected_row >= self.selectable_items.len() {
            self.selected_row = self.selectable_items.len().saturating_sub(1);
        }
        // Skip adapter headers
        while self.selected_row < self.selectable_items.len()
            && self.is_header_row(self.selected_row)
        {
            if self.selected_row + 1 < self.selectable_items.len() {
                self.selected_row += 1;
            } else {
                break;
            }
        }
    }
}

/// Main draw function
pub fn draw(
    frame: &mut Frame,
    adapters: &[AdapterInfo],
    metrics: &MetricsCollector,
    hostname: &str,
    state: &mut AppState,
) {
    state.frame_count += 1;
    state.update_selectable_items(adapters);

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if state.detail_expanded {
            vec![Constraint::Percentage(50), Constraint::Percentage(50)]
        } else {
            vec![Constraint::Min(0)]
        })
        .split(frame.area());

    // Draw main table (always visible)
    draw_main_table(frame, main_layout[0], adapters, metrics, hostname, state);

    // Draw detail panel if expanded
    if state.detail_expanded && main_layout.len() > 1 {
        draw_detail_panel(frame, main_layout[1], adapters, metrics, state);
    }
}

/// Calculate total throughput across all active ports
fn calculate_totals(adapters: &[AdapterInfo], metrics: &MetricsCollector) -> (f64, f64) {
    let mut total_rx = 0.0;
    let mut total_tx = 0.0;
    for adapter in adapters {
        for port in &adapter.ports {
            if let Some(m) = metrics.get_metrics(&adapter.name, port.port_number) {
                total_rx += m.rx_bytes_per_sec;
                total_tx += m.tx_bytes_per_sec;
            }
        }
    }
    (total_rx, total_tx)
}

/// Draw the main table with sparklines
#[allow(clippy::too_many_lines)]
fn draw_main_table(
    frame: &mut Frame,
    area: Rect,
    adapters: &[AdapterInfo],
    metrics: &MetricsCollector,
    hostname: &str,
    state: &AppState,
) {
    // Calculate totals for header
    let (total_rx, total_tx) = calculate_totals(adapters, metrics);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    let mut rows: Vec<Row> = Vec::new();
    let mut row_idx = 0;

    if adapters.is_empty() {
        rows.push(Row::new(vec![
            Cell::from("").style(Style::default()),
            Cell::from("No InfiniBand adapters found").style(Style::default().fg(Color::Yellow)),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ]));
    } else {
        for adapter in adapters {
            // Adapter header row with visual separator
            let is_header_selected = state.selected_row == row_idx;
            let header_style = if is_header_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            };

            rows.push(
                Row::new(vec![
                    Cell::from(""),
                    Cell::from(format!(" {} ", adapter.name)).style(header_style),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                ])
                .height(1),
            );
            row_idx += 1;

            for port in &adapter.ports {
                let is_selected = state.selected_row == row_idx;
                let port_metrics = metrics.get_metrics(&adapter.name, port.port_number);
                let history = metrics.get_history(&adapter.name, port.port_number);

                // State indicator with pulsing effect for active ports
                let (state_str, state_color) = match port.state {
                    PortState::Active => {
                        // Subtle pulse: alternates between bright and dim dot
                        let pulse = if state.frame_count % 60 < 30 {
                            "●"
                        } else {
                            "○"
                        };
                        (format!("{pulse}ACTIVE"), Color::Green)
                    }
                    PortState::Down => ("○DOWN".to_string(), Color::Red),
                    PortState::Unknown => ("?UNKN".to_string(), Color::Yellow),
                };

                // Get throughput values
                let (rx_rate, tx_rate) = if let Some(m) = port_metrics {
                    (
                        format_bytes_per_sec(m.rx_bytes_per_sec),
                        format_bytes_per_sec(m.tx_bytes_per_sec),
                    )
                } else {
                    ("--".to_string(), "--".to_string())
                };

                // Sparkline data (with padding)
                let sparkline_str = if let Some(h) = history {
                    format!(
                        " {} ",
                        render_inline_sparkline(&h.combined_sparkline_data(SPARKLINE_SAMPLES))
                    )
                } else {
                    " ".repeat(SPARKLINE_SAMPLES + 2)
                };

                // Throughput bar (visual indicator of utilization)
                let utilization = if let Some(m) = port_metrics {
                    let max_rate = parse_max_rate(&port.rate);
                    let current_rate = m.rx_bytes_per_sec + m.tx_bytes_per_sec;
                    (current_rate / max_rate * 100.0).min(100.0)
                } else {
                    0.0
                };
                let bar = render_utilization_bar(utilization, 8);

                let row_style = if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                rows.push(
                    Row::new(vec![
                        Cell::from(format!("  {}", port.port_number))
                            .style(Style::default().fg(Color::Cyan)),
                        Cell::from(state_str).style(Style::default().fg(state_color)),
                        Cell::from(truncate_rate(&port.rate)).style(
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::DIM),
                        ),
                        Cell::from(bar),
                        Cell::from(rx_rate).style(Style::default().fg(Color::Blue)),
                        Cell::from(tx_rate).style(Style::default().fg(Color::Magenta)),
                        Cell::from(sparkline_str).style(Style::default().fg(Color::Cyan)),
                        Cell::from(if is_selected { "◀" } else { " " })
                            .style(Style::default().fg(Color::Cyan)),
                    ])
                    .style(row_style)
                    .height(1),
                );
                row_idx += 1;
            }
        }
    }

    let widths = [
        Constraint::Length(4),                            // Port
        Constraint::Length(8),                            // State
        Constraint::Length(12),                           // Link Rate
        Constraint::Length(10),                           // Utilization bar
        Constraint::Length(10),                           // RX Rate
        Constraint::Length(10),                           // TX Rate
        Constraint::Length(SPARKLINE_SAMPLES as u16 + 4), // Sparkline (padded)
        Constraint::Length(2),                            // Selection indicator
    ];

    let header_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);

    let table = Table::new(rows, widths)
        .header(
            Row::new(vec![
                Cell::from("Port").style(header_style),
                Cell::from("State").style(header_style),
                Cell::from("Link").style(header_style),
                Cell::from("Load").style(header_style),
                Cell::from("RX").style(header_style),
                Cell::from("TX").style(header_style),
                Cell::from("History").style(header_style),
                Cell::from("").style(header_style),
            ])
            .height(1)
            .bottom_margin(0),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(Line::from(vec![
                    Span::styled(
                        " ibtop ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("@ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(hostname, Style::default().fg(Color::White)),
                    Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
                    Span::styled("▲ ", Style::default().fg(Color::Green)),
                    Span::styled(
                        format_bytes_per_sec(total_rx),
                        Style::default().fg(Color::Green),
                    ),
                    Span::styled("  ▼ ", Style::default().fg(Color::Blue)),
                    Span::styled(
                        format_bytes_per_sec(total_tx),
                        Style::default().fg(Color::Blue),
                    ),
                    Span::styled(" ", Style::default()),
                ]))
                .title_style(Style::default()),
        );

    frame.render_widget(table, chunks[0]);

    // Help footer - context-sensitive
    let help_spans = if state.detail_expanded {
        vec![
            Span::styled(" ", Style::default().fg(Color::DarkGray)),
            Span::styled("Tab", Style::default().fg(Color::Cyan)),
            Span::styled(" switch tab  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::styled(" close  ", Style::default().fg(Color::DarkGray)),
            Span::styled("j/k", Style::default().fg(Color::Cyan)),
            Span::styled(" select port  ", Style::default().fg(Color::DarkGray)),
            Span::styled("q", Style::default().fg(Color::Cyan)),
            Span::styled(" quit ", Style::default().fg(Color::DarkGray)),
        ]
    } else {
        vec![
            Span::styled(" ", Style::default().fg(Color::DarkGray)),
            Span::styled("j/k", Style::default().fg(Color::Cyan)),
            Span::styled(" navigate  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::styled(" details  ", Style::default().fg(Color::DarkGray)),
            Span::styled("q", Style::default().fg(Color::Cyan)),
            Span::styled(" quit ", Style::default().fg(Color::DarkGray)),
        ]
    };

    let help = Paragraph::new(Line::from(help_spans));
    frame.render_widget(help, chunks[1]);
}

/// Draw the detail panel with charts
fn draw_detail_panel(
    frame: &mut Frame,
    area: Rect,
    adapters: &[AdapterInfo],
    metrics: &MetricsCollector,
    state: &AppState,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Line::from(vec![Span::styled(
            " Detail View ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));

    // Get selected port info
    let selected = state.selected_port();
    if selected.is_none() {
        let msg = Paragraph::new("Select a port to view details")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        frame.render_widget(msg, area);
        return;
    }

    let (adapter_name, port_num) = selected.unwrap();
    let port_info = adapters
        .iter()
        .find(|a| a.name == adapter_name)
        .and_then(|a| a.ports.iter().find(|p| p.port_number == port_num));

    let history = metrics.get_history(adapter_name, port_num);
    let current_metrics = metrics.get_metrics(adapter_name, port_num);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Layout for detail panel
    let detail_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Tab bar
            Constraint::Length(3), // Stats summary
            Constraint::Min(0),    // Chart area
        ])
        .split(inner);

    // Tab bar
    let tabs = Tabs::new(vec!["Throughput", "Packets", "Errors"])
        .select(state.detail_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::raw(" | "));

    frame.render_widget(tabs, detail_layout[0]);

    // Stats summary
    if let (Some(port), Some(m)) = (port_info, current_metrics) {
        let stats_line = Line::from(vec![
            Span::styled(
                format!("{adapter_name}:"),
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                format!("{port_num} ", port_num = port.port_number),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(
                format!("{} ", port.state),
                Style::default().fg(match port.state {
                    PortState::Active => Color::Green,
                    PortState::Down => Color::Red,
                    PortState::Unknown => Color::Yellow,
                }),
            ),
            Span::styled("| ", Style::default().fg(Color::DarkGray)),
            Span::styled("RX: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format_bytes_per_sec(m.rx_bytes_per_sec),
                Style::default().fg(Color::Blue),
            ),
            Span::styled(" TX: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format_bytes_per_sec(m.tx_bytes_per_sec),
                Style::default().fg(Color::Magenta),
            ),
        ]);

        let stats_para = Paragraph::new(stats_line);
        frame.render_widget(stats_para, detail_layout[1]);
    }

    // Chart area
    if let Some(h) = history {
        draw_chart(frame, detail_layout[2], h, state.detail_tab);
    } else {
        let msg = Paragraph::new("Collecting data...").style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, detail_layout[2]);
    }
}

/// Auto-scale throughput value and return scaled value with unit
fn auto_scale_throughput(bytes_per_sec: f64) -> (f64, &'static str) {
    if bytes_per_sec >= 1_000_000_000.0 {
        (bytes_per_sec / 1_000_000_000.0, "GB/s")
    } else if bytes_per_sec >= 1_000_000.0 {
        (bytes_per_sec / 1_000_000.0, "MB/s")
    } else if bytes_per_sec >= 1_000.0 {
        (bytes_per_sec / 1_000.0, "KB/s")
    } else {
        (bytes_per_sec, "B/s")
    }
}

/// Draw a chart based on the selected tab
#[allow(clippy::too_many_lines)]
fn draw_chart(frame: &mut Frame, area: Rect, history: &PortHistory, tab: usize) {
    // First, find the max value to determine scale
    let (rx_raw, tx_raw): (Vec<f64>, Vec<f64>) = match tab {
        0 => (
            history.rx_bytes_per_sec.iter().copied().collect(),
            history.tx_bytes_per_sec.iter().copied().collect(),
        ),
        1 => (
            history.rx_packets_per_sec.iter().copied().collect(),
            history.tx_packets_per_sec.iter().copied().collect(),
        ),
        _ => {
            let errors: Vec<f64> = history.error_rate.iter().copied().collect();
            (errors.clone(), errors)
        }
    };

    if rx_raw.is_empty() {
        return;
    }

    let max_raw = rx_raw
        .iter()
        .chain(tx_raw.iter())
        .copied()
        .fold(0.0_f64, f64::max)
        .max(0.001); // Avoid division by zero

    // Determine scale and unit based on max value
    let (divisor, y_label) = match tab {
        0 => {
            // Throughput - auto-scale
            let (_, unit) = auto_scale_throughput(max_raw);
            let div = match unit {
                "GB/s" => 1_000_000_000.0,
                "MB/s" => 1_000_000.0,
                "KB/s" => 1_000.0,
                _ => 1.0,
            };
            (div, unit)
        }
        1 => {
            // Packets - scale to K or M
            if max_raw >= 1_000_000.0 {
                (1_000_000.0, "Mpps")
            } else if max_raw >= 1_000.0 {
                (1_000.0, "Kpps")
            } else {
                (1.0, "pps")
            }
        }
        _ => (1.0, "err/s"),
    };

    // Scale the data
    let rx_data: Vec<(f64, f64)> = rx_raw
        .iter()
        .enumerate()
        .map(|(i, v)| (i as f64, v / divisor))
        .collect();
    let tx_data: Vec<(f64, f64)> = tx_raw
        .iter()
        .enumerate()
        .map(|(i, v)| (i as f64, v / divisor))
        .collect();

    let max_scaled = max_raw / divisor;
    let x_max = rx_data.len() as f64;

    // Colors
    let (rx_color, tx_color) = match tab {
        0 => (Color::Blue, Color::Magenta),
        1 => (Color::Green, Color::Yellow),
        _ => (Color::Red, Color::Red),
    };

    let datasets = if tab == 2 {
        vec![Dataset::default()
            .name("Errors")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(rx_color))
            .data(&rx_data)]
    } else {
        vec![
            Dataset::default()
                .name("RX")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(rx_color))
                .data(&rx_data),
            Dataset::default()
                .name("TX")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(tx_color))
                .data(&tx_data),
        ]
    };

    // Time label based on data points (4 samples/sec = 250ms each)
    let time_span_secs = rx_data.len() as f64 * 0.25;
    let time_label = if time_span_secs >= 60.0 {
        let mins = time_span_secs / 60.0;
        format!("{mins:.0}m ago")
    } else {
        format!("{time_span_secs:.0}s ago")
    };

    let chart = Chart::new(datasets)
        .x_axis(
            Axis::default()
                .style(Style::default().fg(Color::DarkGray))
                .bounds([0.0, x_max])
                .labels(vec![
                    Span::styled(time_label, Style::default().fg(Color::DarkGray)),
                    Span::styled("now", Style::default().fg(Color::White)),
                ]),
        )
        .y_axis(
            Axis::default()
                .title(y_label)
                .style(Style::default().fg(Color::DarkGray))
                .bounds([0.0, max_scaled * 1.1])
                .labels(vec![
                    Span::raw("0"),
                    Span::styled(
                        format!("{max_scaled:.1}"),
                        Style::default().fg(Color::White),
                    ),
                ]),
        );

    frame.render_widget(chart, area);
}

/// Render an inline sparkline as Unicode characters
fn render_inline_sparkline(data: &[u64]) -> String {
    const SPARK_CHARS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    if data.is_empty() {
        return String::new();
    }

    data.iter()
        .map(|&v| SPARK_CHARS[(v as usize).min(7)])
        .collect()
}

/// Render a utilization bar
fn render_utilization_bar(percent: f64, width: usize) -> String {
    let filled = ((percent / 100.0) * width as f64).round() as usize;
    let filled = filled.min(width);

    (0..width)
        .map(|i| if i < filled { '█' } else { '░' })
        .collect()
}

/// Parse max rate from rate string (e.g., "100 Gb/sec" -> bytes/sec)
fn parse_max_rate(rate_str: &str) -> f64 {
    // Extract the number and unit
    let parts: Vec<&str> = rate_str.split_whitespace().collect();
    if parts.len() >= 2 {
        if let Ok(num) = parts[0].parse::<f64>() {
            // Convert Gb/sec to bytes/sec
            return num * 1_000_000_000.0 / 8.0;
        }
    }
    // Default to 100 Gbps
    12_500_000_000.0
}

/// Truncate rate string for display
fn truncate_rate(rate: &str) -> String {
    // Extract just the speed part (e.g., "100 Gb/sec")
    let parts: Vec<&str> = rate.split('(').collect();
    if parts.is_empty() {
        rate.to_string()
    } else {
        parts[0].trim().to_string()
    }
}

#[allow(dead_code)] // Kept for API completeness
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    let mut value = bytes;
    let mut unit_index = 0;

    while value >= 1024 && unit_index < UNITS.len() - 1 {
        value /= 1024;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{}{}", value, UNITS[unit_index])
    } else {
        let fractional = (bytes >> (10 * (unit_index - 1))) % 1024;
        let decimal_part = (fractional * 10) / 1024;
        format!("{}.{}{}", value, decimal_part, UNITS[unit_index])
    }
}

pub fn format_bytes_per_sec(bytes_per_sec: f64) -> String {
    const UNITS: &[&str] = &["B/s", "KB/s", "MB/s", "GB/s", "TB/s"];
    let mut value = bytes_per_sec;
    let mut unit_index = 0;

    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    if value < 0.1 {
        format!("{:.2}{}", value, UNITS[unit_index])
    } else {
        format!("{:.1}{}", value, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0B");
        assert_eq!(format_bytes(1023), "1023B");
        assert_eq!(format_bytes(1024), "1.0KB");
        assert_eq!(format_bytes(1025), "1.0KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0GB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 1024), "1.0TB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 1024 * 1024), "1.0PB");
    }

    #[test]
    fn test_format_bytes_per_sec() {
        assert_eq!(format_bytes_per_sec(0.0), "0.00B/s");
        assert_eq!(format_bytes_per_sec(1023.0), "1023.0B/s");
        assert_eq!(format_bytes_per_sec(1024.0), "1.0KB/s");
        assert_eq!(format_bytes_per_sec(1025.0), "1.0KB/s");
        assert_eq!(format_bytes_per_sec(1024.0 * 1024.0), "1.0MB/s");
        assert_eq!(format_bytes_per_sec(1024.0 * 1024.0 * 1024.0), "1.0GB/s");
        assert_eq!(
            format_bytes_per_sec(1024.0 * 1024.0 * 1024.0 * 1024.0),
            "1.0TB/s"
        );
    }

    #[test]
    fn test_render_inline_sparkline() {
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let result = render_inline_sparkline(&data);
        assert_eq!(result, "▁▂▃▄▅▆▇█");
    }

    #[test]
    fn test_render_inline_sparkline_empty() {
        let data: Vec<u64> = vec![];
        let result = render_inline_sparkline(&data);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_max_rate() {
        assert!((parse_max_rate("100 Gb/sec (4X EDR)") - 12_500_000_000.0).abs() < 1.0);
        assert!((parse_max_rate("200 Gb/sec") - 25_000_000_000.0).abs() < 1.0);
        assert!((parse_max_rate("invalid") - 12_500_000_000.0).abs() < 1.0); // Default
    }

    #[test]
    fn test_truncate_rate() {
        assert_eq!(truncate_rate("100 Gb/sec (4X EDR)"), "100 Gb/sec");
        assert_eq!(truncate_rate("200 Gb/sec"), "200 Gb/sec");
    }

    #[test]
    fn test_utilization_bar() {
        let bar = render_utilization_bar(50.0, 10);
        // Unicode chars are multi-byte, so count chars not bytes
        assert_eq!(bar.chars().count(), 10);
        assert!(bar.contains('█'));
        assert!(bar.contains('░'));
    }

    #[test]
    fn test_app_state_navigation() {
        let mut state = AppState::new();
        state.selectable_items = vec![
            None,
            Some(("mlx5_0".to_string(), 1)),
            Some(("mlx5_0".to_string(), 2)),
            None,
            Some(("mlx5_1".to_string(), 1)),
        ];
        state.selected_row = 1;

        state.select_next();
        assert_eq!(state.selected_row, 2);

        state.select_next();
        // Should skip the None at index 3
        assert_eq!(state.selected_row, 4);

        state.select_prev();
        assert_eq!(state.selected_row, 2);
    }

    #[test]
    fn test_app_state_toggle_detail() {
        let mut state = AppState::new();
        assert!(!state.detail_expanded);

        state.toggle_detail();
        assert!(state.detail_expanded);

        state.toggle_detail();
        assert!(!state.detail_expanded);
    }

    #[test]
    fn test_app_state_tab_cycling() {
        let mut state = AppState::new();
        assert_eq!(state.detail_tab, 0);

        state.next_tab();
        assert_eq!(state.detail_tab, 1);

        state.next_tab();
        assert_eq!(state.detail_tab, 2);

        state.next_tab();
        assert_eq!(state.detail_tab, 0);

        state.prev_tab();
        assert_eq!(state.detail_tab, 2);
    }
}
