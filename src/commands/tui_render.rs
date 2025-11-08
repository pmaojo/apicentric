//! Rendering functions for TUI panels
//! 
//! This module is only available when the `tui` feature is enabled.

#![cfg(feature = "tui")]

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::tui_state::{ServiceListState, TuiAppState};

/// Render the service list panel
pub fn render_service_list(f: &mut Frame, area: Rect, state: &ServiceListState) {
    let items: Vec<ListItem> = state
        .items
        .iter()
        .enumerate()
        .map(|(idx, service)| {
            // Choose indicator based on running status
            let indicator = if service.is_running { "●" } else { "○" };
            let indicator_color = if service.is_running {
                Color::Green
            } else {
                Color::Gray
            };

            // Format the service line
            let content = format!(
                "{} {} :{}",
                indicator, service.name, service.port
            );

            // Add request count if available
            let content = if service.request_count > 0 {
                format!("{} ({})", content, service.request_count)
            } else {
                content
            };

            // Create the list item with appropriate styling
            let style = if idx == state.selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled(indicator, Style::default().fg(indicator_color)),
                Span::styled(
                    format!(" {} :{}", service.name, service.port),
                    style,
                ),
                if service.request_count > 0 {
                    Span::styled(
                        format!(" ({})", service.request_count),
                        Style::default().fg(Color::Cyan),
                    )
                } else {
                    Span::raw("")
                },
            ]);

            ListItem::new(line)
        })
        .collect();

    // Create title with count
    let title = format!(
        " Services ({}/{}) ",
        if state.items.is_empty() {
            0
        } else {
            state.selected + 1
        },
        state.items.len()
    );

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White)),
    );

    f.render_widget(list, area);
}

/// Render the log view panel
pub fn render_log_view(f: &mut Frame, area: Rect, state: &TuiAppState) {
    let filtered = state.logs.filtered_entries();
    let total_count = filtered.len();

    // Calculate visible range based on scroll
    let visible_height = area.height.saturating_sub(2) as usize; // Account for borders
    let scroll = state.logs.scroll.min(total_count.saturating_sub(1));

    let items: Vec<ListItem> = filtered
        .iter()
        .skip(scroll)
        .take(visible_height)
        .map(|entry| {
            // Format timestamp
            let timestamp = entry.timestamp.format("%H:%M:%S").to_string();

            // Choose color based on status code
            let status_color = match entry.status {
                200..=299 => Color::Green,
                300..=399 => Color::Yellow,
                400..=499 => Color::Red,
                500..=599 => Color::Magenta,
                _ => Color::White,
            };

            // Format the log line
            let line = Line::from(vec![
                Span::styled(
                    format!("{} ", timestamp),
                    Style::default().fg(Color::Gray),
                ),
                Span::styled(
                    format!("{:6} ", entry.method),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:30} ", entry.path),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("→ {}", entry.status),
                    Style::default().fg(status_color).add_modifier(Modifier::BOLD),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    // Create title with scroll indicator
    let title = if total_count > 0 {
        format!(
            " Request Logs ({}-{} of {}) ",
            scroll + 1,
            (scroll + visible_height).min(total_count),
            total_count
        )
    } else {
        " Request Logs (0) ".to_string()
    };

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White)),
    );

    f.render_widget(list, area);
}

/// Render the actions panel
pub fn render_actions_panel(f: &mut Frame, area: Rect, state: &TuiAppState) {
    let mut lines = vec![
        Line::from(vec![
            Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(": Quit"),
        ]),
        Line::from(vec![
            Span::styled("↑↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(": Navigate"),
        ]),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(": Start/Stop"),
        ]),
        Line::from(vec![
            Span::styled("f", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(": Filter"),
        ]),
        Line::from(vec![
            Span::styled("r", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(": Refresh"),
        ]),
        Line::from(vec![
            Span::styled("c", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(": Clear logs"),
        ]),
        Line::from(vec![
            Span::styled("s", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(": Save logs"),
        ]),
        Line::from(vec![
            Span::styled("/", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(": Search"),
        ]),
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(": Switch panel"),
        ]),
        Line::from(vec![
            Span::styled("?", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(": Help"),
        ]),
        Line::from(""),
    ];

    // Show filter status if active
    if state.logs.filter.is_active() {
        lines.push(Line::from(vec![
            Span::styled("Filter: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(
                state.logs.filter.description(),
                Style::default().fg(Color::White),
            ),
        ]));
        lines.push(Line::from(""));
    }

    // Show status message if present
    if let Some(ref msg) = state.status_message {
        lines.push(Line::from(vec![
            Span::styled("✓ ", Style::default().fg(Color::Green)),
            Span::styled(msg, Style::default().fg(Color::Green)),
        ]));
    }

    // Show error message if present
    if let Some(ref msg) = state.error_message {
        lines.push(Line::from(vec![
            Span::styled("✗ ", Style::default().fg(Color::Red)),
            Span::styled(msg, Style::default().fg(Color::Red)),
        ]));
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(" Actions ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White)),
    );

    f.render_widget(paragraph, area);
}
