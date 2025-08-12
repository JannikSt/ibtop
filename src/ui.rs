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
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(frame.size());

    draw_title(frame, chunks[0]);
    draw_adapters(frame, chunks[1], adapters);
}

fn draw_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("ibtop - InfiniBand Monitor")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, area);
}

fn draw_adapters(frame: &mut Frame, area: Rect, adapters: &[AdapterInfo]) {
    let mut items: Vec<ListItem> = Vec::new();
    
    items.push(ListItem::new(""));
    items.push(ListItem::new(Line::from(vec![
        Span::styled(
            "    Port      State       │   RX Data           │   TX Data           ",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::UNDERLINED),
        ),
    ])));
    items.push(ListItem::new(""));
    
    if adapters.is_empty() {
        items.push(ListItem::new(Line::from(vec![Span::styled(
            "    No InfiniBand adapters found",
            Style::default().fg(Color::Yellow),
        )])));
        items.push(ListItem::new(""));
    } else {
        for adapter in adapters {
            items.push(ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled("Adapter: ", Style::default().fg(Color::Green)),
                Span::styled(
                    &adapter.name,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ])));
            items.push(ListItem::new(""));

            for port in &adapter.ports {
                let state_color = if port.state == "ACTIVE" {
                    Color::Green
                } else {
                    Color::Red
                };
                
                let state_formatted = format!("{:<10}", port.state);
                
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("    Port "),
                    Span::styled(
                        format!("{:<3}", port.port_number),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::raw("   "),
                    Span::styled(state_formatted, Style::default().fg(state_color)),
                    Span::raw(" │  "),
                    Span::styled(
                        format_bytes(port.counters.rx_bytes),
                        Style::default().fg(Color::Blue),
                    ),
                    Span::raw("     │  "),
                    Span::styled(
                        format_bytes(port.counters.tx_bytes),
                        Style::default().fg(Color::Blue),
                    ),
                ])));
            }

            items.push(ListItem::new(""));
            items.push(ListItem::new(""));
        }
    }

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("InfiniBand Adapters"));
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

    format!("{:>10.2} {:<2}", value, UNITS[unit_index])
}
