use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::metrics::MetricsCollector;
use crate::types::AdapterInfo;

pub fn draw(frame: &mut Frame, adapters: &[AdapterInfo], metrics: &MetricsCollector) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.size());

    draw_adapters(frame, chunks[1], adapters, metrics);
    draw_help_footer(frame, chunks[2]);
}

fn draw_help_footer(frame: &mut Frame, area: Rect) {
    let help_text = Paragraph::new("Controls: q=quit").style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help_text, area);
}

fn draw_adapters(
    frame: &mut Frame,
    area: Rect,
    adapters: &[AdapterInfo],
    metrics: &MetricsCollector,
) {
    let mut rows: Vec<Row> = Vec::new();

    if adapters.is_empty() {
        rows.push(Row::new(vec![
            Cell::from("No InfiniBand adapters found").style(Style::default().fg(Color::Yellow))
        ]));
    } else {
        for adapter in adapters {
            // Add adapter header row that spans the full width
            rows.push(Row::new(vec![
                Cell::from("Adapter:"),
                Cell::from(adapter.name.to_string()).style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
            ]));

            for port in &adapter.ports {
                let state_color = match port.state {
                    crate::types::PortState::Active => Color::Green,
                    crate::types::PortState::Down => Color::Red,
                    crate::types::PortState::Unknown => Color::Yellow,
                };

                let port_metrics = metrics.get_metrics(&adapter.name, port.port_number);
                let (rx_rate, tx_rate) = if let Some(metrics) = port_metrics {
                    (
                        format_bytes_per_sec(metrics.rx_bytes_per_sec),
                        format_bytes_per_sec(metrics.tx_bytes_per_sec),
                    )
                } else {
                    ("--".to_string(), "--".to_string())
                };

                rows.push(Row::new(vec![
                    Cell::from(format!("{}", port.port_number))
                        .style(Style::default().fg(Color::Cyan)),
                    Cell::from(port.state.to_string()).style(Style::default().fg(state_color)),
                    Cell::from(port.rate.clone()).style(Style::default().fg(Color::White)),
                    Cell::from(format_bytes(port.counters.rx_bytes))
                        .style(Style::default().fg(Color::Blue)),
                    Cell::from(format_bytes(port.counters.tx_bytes))
                        .style(Style::default().fg(Color::Blue)),
                    Cell::from(rx_rate).style(Style::default().fg(Color::Magenta)),
                    Cell::from(tx_rate).style(Style::default().fg(Color::Magenta)),
                ]));
            }
        }
    }

    let widths = [
        Constraint::Length(8),  // Port
        Constraint::Length(10), // State
        Constraint::Length(12), // Rate
        Constraint::Length(12), // RX Data
        Constraint::Length(12), // TX Data
        Constraint::Length(12), // RX Rate
        Constraint::Length(12), // TX Rate
    ];

    let table = Table::new(rows, widths)
        .header(Row::new(vec![
            Cell::from("Port").style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Cell::from("State").style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Cell::from("Rate").style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Cell::from("RX Data").style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Cell::from("TX Data").style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Cell::from("RX Rate").style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Cell::from("TX Rate").style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("ibtop - InfiniBand Monitor"),
        );

    frame.render_widget(table, area);
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit_index = 0;

    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1}{}", value, UNITS[unit_index])
}

fn format_bytes_per_sec(bytes_per_sec: f64) -> String {
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
