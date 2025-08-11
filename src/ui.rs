use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::types::AdapterInfo;

pub fn draw(frame: &mut Frame, adapters: &[AdapterInfo]) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(frame.size());

    draw_header(frame, chunks[0]);
    draw_adapters(frame, chunks[1], adapters);
}

fn draw_header(frame: &mut Frame, area: Rect) {
    let header = Paragraph::new("InfiniBand Top - Simple Monitor")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL).title("ibtop"));
    frame.render_widget(header, area);
}

fn draw_adapters(frame: &mut Frame, area: Rect, adapters: &[AdapterInfo]) {
    let items: Vec<ListItem> = if adapters.is_empty() {
        vec![ListItem::new(Line::from(vec![Span::styled(
            "No InfiniBand adapters found",
            Style::default().fg(Color::Yellow),
        )]))]
    } else {
        adapters
            .iter()
            .flat_map(|adapter| {
                let mut items = vec![ListItem::new(Line::from(vec![
                    Span::styled("Adapter: ", Style::default().fg(Color::Green)),
                    Span::styled(
                        &adapter.name,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]))];

                for port in &adapter.ports {
                    let state_color = if port.state == "ACTIVE" {
                        Color::Green
                    } else {
                        Color::Red
                    };
                    items.push(ListItem::new(Line::from(vec![
                        Span::raw("  Port "),
                        Span::styled(
                            port.port_number.to_string(),
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::raw(": "),
                        Span::styled(&port.state, Style::default().fg(state_color)),
                        Span::raw(" | RX: "),
                        Span::styled(
                            format_bytes(port.counters.rx_bytes),
                            Style::default().fg(Color::Blue),
                        ),
                        Span::raw(" | TX: "),
                        Span::styled(
                            format_bytes(port.counters.tx_bytes),
                            Style::default().fg(Color::Blue),
                        ),
                    ])));
                }

                items.push(ListItem::new(""));
                items
            })
            .collect()
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Adapters & Ports"),
    );
    frame.render_widget(list, area);
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit_index = 0;

    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", value, UNITS[unit_index])
}
