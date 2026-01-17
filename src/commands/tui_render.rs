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

use std::time::Duration;

use super::tui_state::{ServiceListState, TuiAppState};

/// Helper function to calculate centered dialog area
fn calculate_dialog_area(f: &Frame, width: u16, height: u16) -> Rect {
    let area = f.size();
    let dialog_width = width.min(area.width.saturating_sub(4));
    let dialog_height = height;

    Rect {
        x: (area.width.saturating_sub(dialog_width)) / 2,
        y: (area.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    }
}

/// Helper function to create a styled action line
fn create_action_line<'a>(key: &'a str, description: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            key,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(description),
    ])
}

/// Render the service list panel
pub fn render_service_list(f: &mut Frame, area: Rect, state: &ServiceListState, is_focused: bool) {
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

            // Create the list item with appropriate styling
            let style = if idx == state.selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let mut spans = Vec::with_capacity(3);
            spans.push(Span::styled(
                indicator,
                Style::default().fg(indicator_color),
            ));
            spans.push(Span::styled(
                format!(" {} :{}", service.name, service.port),
                style,
            ));

            if service.request_count > 0 {
                spans.push(Span::styled(
                    format!(" ({})", service.request_count),
                    Style::default().fg(Color::Cyan),
                ));
            }

            let line = Line::from(spans);

            ListItem::new(line)
        })
        .collect();

    // Create title with count and focus indicator
    let title = format!(
        "{} Services ({}/{}) ",
        if is_focused { "▶" } else { " " },
        if state.items.is_empty() {
            0
        } else {
            state.selected + 1
        },
        state.items.len()
    );

    let border_color = if is_focused {
        Color::Yellow
    } else {
        Color::White
    };

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );

    f.render_widget(list, area);
}

/// Render the log view panel
pub fn render_log_view(f: &mut Frame, area: Rect, state: &TuiAppState, is_focused: bool) {
    let filtered = state.logs.filtered_entries();
    let total_count = filtered.len();

    // Calculate visible range based on scroll
    let visible_height = area.height.saturating_sub(2) as usize; // Account for borders
    let scroll = state.logs.scroll.min(total_count.saturating_sub(1));

    // Get only the visible items for rendering
    let visible_start = scroll;
    let visible_end = (scroll + visible_height).min(total_count);

    let items: Vec<ListItem> = filtered[visible_start..visible_end]
        .iter()
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
                Span::styled(format!("{} ", timestamp), Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:6} ", entry.method),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:30} ", entry.path),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("→ {}", entry.status),
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    // Create title with scroll indicator and focus indicator
    let title = if total_count > 0 {
        format!(
            "{} Request Logs ({}-{} of {}) ",
            if is_focused { "▶" } else { " " },
            scroll + 1,
            (scroll + visible_height).min(total_count),
            total_count
        )
    } else {
        format!("{} Request Logs (0) ", if is_focused { "▶" } else { " " })
    };

    let border_color = if is_focused {
        Color::Yellow
    } else {
        Color::White
    };

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );

    f.render_widget(list, area);
}

/// Render the filter dialog
pub fn render_filter_dialog(f: &mut Frame, state: &TuiAppState) {
    use ratatui::{layout::Alignment, widgets::Clear};

    let dialog_area = calculate_dialog_area(f, 60, 7);

    // Clear the area behind the dialog
    f.render_widget(Clear, dialog_area);

    // Create dialog content
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("Format: "),
            Span::styled("method:GET", Style::default().fg(Color::Cyan)),
            Span::raw(", "),
            Span::styled("status:200", Style::default().fg(Color::Cyan)),
            Span::raw(", "),
            Span::styled("service:api", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Yellow)),
            Span::raw(state.input.value()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(" to apply, "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(" to cancel"),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Filter Logs ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .alignment(Alignment::Left);

    f.render_widget(paragraph, dialog_area);
}

/// Render the search dialog
pub fn render_search_dialog(f: &mut Frame, state: &TuiAppState) {
    use ratatui::{layout::Alignment, widgets::Clear};

    let dialog_area = calculate_dialog_area(f, 50, 6);

    // Clear the area behind the dialog
    f.render_widget(Clear, dialog_area);

    // Create dialog content
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Yellow)),
            Span::raw(state.input.value()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(" to search, "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(" to cancel"),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Search Logs ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .alignment(Alignment::Left);

    f.render_widget(paragraph, dialog_area);
}

/// Render the help dialog
pub fn render_help_dialog(f: &mut Frame) {
    use ratatui::{layout::Alignment, widgets::Clear};

    let dialog_area = calculate_dialog_area(f, 60, 18);

    // Clear the area behind the dialog
    f.render_widget(Clear, dialog_area);

    // Create help content
    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  ↑/↓       Navigate focused panel"),
        Line::from("  PgUp/PgDn Scroll logs"),
        Line::from("  Tab       Switch panel focus"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Actions",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  Enter     Start/Stop service"),
        Line::from("  f         Open filter dialog"),
        Line::from("  r         Refresh status"),
        Line::from("  c         Clear logs"),
        Line::from("  s         Save logs to file"),
        Line::from("  /         Search logs"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Other",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  ?         Show this help"),
        Line::from("  q         Quit"),
        Line::from(""),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .alignment(Alignment::Left);

    f.render_widget(paragraph, dialog_area);
}

// The small shim `render_actions_panel` was removed to avoid dead_code allowances.

/// Render the actions panel with performance metrics
pub fn render_actions_panel_with_metrics(
    f: &mut Frame,
    area: Rect,
    state: &TuiAppState,
    avg_input_latency: Option<Duration>,
    max_input_latency: Option<Duration>,
) {
    let mut lines = vec![
        create_action_line("q", ": Quit"),
        create_action_line("↑↓", ": Nav"),
        create_action_line("Enter", ": Toggle"),
        create_action_line("f", ": Filter"),
        create_action_line("r", ": Refresh"),
        create_action_line("c", ": Clear"),
        create_action_line("s", ": Save"),
        create_action_line("/", ": Search"),
        create_action_line("Tab", ": Switch"),
        create_action_line("?", ": Help"),
        create_action_line("m", ": Market"),
        Line::from(""), // Spacer
    ];

    // Show Dashboard hint
    let dashboard_hint = if state.dashboard.active {
        "d: Logs"
    } else {
        "d: Dashboard"
    };
    lines.push(Line::from(vec![Span::styled(dashboard_hint, Style::default().fg(Color::Cyan))]));
    lines.push(Line::from(""));

    // Show filter status if active
    if state.logs.filter.is_active() {
        lines.push(Line::from(vec![
            Span::styled(
                "Filter: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                state.logs.filter.description(),
                Style::default().fg(Color::White),
            ),
        ]));
        lines.push(Line::from(""));
    }

    // Show loading indicator if active
    if state.is_loading {
        lines.push(Line::from(vec![
            Span::styled("⟳ ", Style::default().fg(Color::Yellow)),
            Span::styled("Loading...", Style::default().fg(Color::Yellow)),
        ]));
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

    // Show performance metrics if available
    if let (Some(avg), Some(max)) = (avg_input_latency, max_input_latency) {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "Performance",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]));
        let avg_ms = avg.as_millis();
        let max_ms = max.as_millis();
        let avg_color = if avg_ms < 100 {
            Color::Green
        } else {
            Color::Yellow
        };
        let max_color = if max_ms < 100 {
            Color::Green
        } else {
            Color::Red
        };
        lines.push(Line::from(vec![
            Span::raw("Avg latency: "),
            Span::styled(format!("{}ms", avg_ms), Style::default().fg(avg_color)),
        ]));
        lines.push(Line::from(vec![
            Span::raw("Max latency: "),
            Span::styled(format!("{}ms", max_ms), Style::default().fg(max_color)),
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

/// Render the retro telemetry dashboard
pub fn render_dashboard_view(f: &mut Frame, area: Rect, state: &TuiAppState) {
    use ratatui::widgets::{Sparkline, Gauge};
    use ratatui::layout::{Constraint, Direction, Layout};

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Telemetry Dashboard ")
        .border_type(ratatui::widgets::BorderType::Double) // Retro look
        .border_style(Style::default().fg(Color::Cyan));

    f.render_widget(block.clone(), area);

    // Inner area for grid
    let inner_area = block.inner(area);

    // Get running services
    let running_services: Vec<_> = state.services.items.iter()
        .filter(|s| s.is_running)
        .collect();
    
    if running_services.is_empty() {
        let text = Paragraph::new("No running services...")
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(text, inner_area);
        return;
    }

    // Create grid layout (simple rows for now)
    let rows = running_services.len().max(1);
    let constraints: Vec<Constraint> = (0..rows).map(|_| Constraint::Length(4)).collect(); // 4 lines per service
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner_area);

    for (i, service) in running_services.iter().enumerate() {
        if i >= chunks.len() { break; }
        
        let metrics_opt = state.dashboard.metrics.get(&service.name);

        // Service Row Layout: Name/Status | Sparkline | Stats
        let row_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(chunks[i]);

        // 1. Name
        let name_widget = Paragraph::new(Line::from(vec![
            Span::styled("● ", Style::default().fg(Color::Green)), // LED
            Span::styled(service.name.clone(), Style::default().add_modifier(Modifier::BOLD)),
        ]))
        .block(Block::default().borders(Borders::RIGHT).border_style(Style::default().fg(Color::DarkGray)));
        f.render_widget(name_widget, row_chunks[0]);

        // 2. Sparkline
        let empty_data = vec![0u64; 50];
        let data: Vec<u64> = metrics_opt
            .map(|m| m.request_history.iter().copied().collect())
            .unwrap_or_else(|| empty_data.clone());
            
        // Determine max for scaling
        let max = data.iter().max().copied().unwrap_or(1).max(1);
        
        let sparkline = Sparkline::default()
            .block(Block::default().title("Activity").borders(Borders::NONE))
            .data(&data)
            .max(max)
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(sparkline, row_chunks[1]);

        // 3. Stats (Gauge or Value)
        let last_val = metrics_opt.and_then(|m| m.request_history.back().copied()).unwrap_or(0);
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::LEFT).border_style(Style::default().fg(Color::DarkGray)))
            .gauge_style(Style::default().fg(Color::Red))
            .ratio( (last_val as f64 / (max as f64).max(1.0)).min(1.0) )
            .label(format!("{} rps", last_val)); 
        f.render_widget(gauge, row_chunks[2]);
    }
}

/// Render the marketplace dialog
pub fn render_marketplace_dialog(f: &mut Frame, state: &TuiAppState) {
    use ratatui::{layout::Alignment, widgets::Clear};

    let dialog_area = calculate_dialog_area(f, 70, 20);

    // Clear the area behind the dialog
    f.render_widget(Clear, dialog_area);

    let mut lines = vec![Line::from("")];
    
    for (idx, item) in state.marketplace.items.iter().enumerate() {
        let is_selected = idx == state.marketplace.selected;
        
        let indicator = if is_selected { "▶ " } else { "  " };
        let style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        
        let category_color = match item.category.as_str() {
            "SaaS" => Color::Cyan,
            "Device" => Color::Green,
            "IoT Twin" => Color::Magenta,
            "Template" => Color::Blue,
            _ => Color::White,
        };
        
        lines.push(Line::from(vec![
            Span::styled(indicator, style),
            Span::styled(
                format!("[{}] ", item.category),
                Style::default().fg(category_color),
            ),
            Span::styled(&item.name, style),
        ]));
    }
    
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(" to install, "),
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(" to navigate, "),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::raw(" to close"),
    ]));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(format!(
                    " 🛒 Marketplace ({}/{}) ",
                    state.marketplace.selected + 1,
                    state.marketplace.items.len()
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .alignment(Alignment::Left);

    f.render_widget(paragraph, dialog_area);
}

