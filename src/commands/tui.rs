use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use apicentric::{ApicentricError, ApicentricResult};

/// Launch a simple terminal dashboard with service list, logs and actions panes.
///
/// The interface exits gracefully when `Ctrl+C` or `q` is pressed.
pub fn tui_command() -> ApicentricResult<()> {
    enable_raw_mode().map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to enable raw mode: {e}"), None::<String>)
    })?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to initialize terminal: {e}"),
            None::<String>,
        )
    })?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to create terminal backend: {e}"),
            None::<String>,
        )
    })?;

    let res = run_app(&mut terminal);

    disable_raw_mode().map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to disable raw mode: {e}"), None::<String>)
    })?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to restore terminal: {e}"), None::<String>)
    })?;
    terminal.show_cursor().map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to show cursor: {e}"), None::<String>)
    })?;

    res
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> ApicentricResult<()> {
    loop {
        terminal
            .draw(|f| {
                let size = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(25),
                        Constraint::Percentage(50),
                        Constraint::Percentage(25),
                    ])
                    .split(size);

                let services = ["service-a", "service-b", "service-c"]
                    .iter()
                    .map(|s| ListItem::new(*s))
                    .collect::<Vec<_>>();
                let service_list = List::new(services)
                    .block(Block::default().title("Services").borders(Borders::ALL));
                f.render_widget(service_list, chunks[0]);

                let logs = List::new(vec![ListItem::new("Awaiting logs...")])
                    .block(Block::default().title("Logs").borders(Borders::ALL));
                f.render_widget(logs, chunks[1]);

                let actions = Paragraph::new("q: quit\nCtrl+C: exit")
                    .block(Block::default().title("Actions").borders(Borders::ALL));
                f.render_widget(actions, chunks[2]);
            })
            .map_err(|e| ApicentricError::runtime_error(format!("Render error: {e}"), None::<String>))?;

        if event::poll(Duration::from_millis(250)).map_err(|e| {
            ApicentricError::runtime_error(format!("Event poll failed: {e}"), None::<String>)
        })? {
            match event::read().map_err(|e| {
                ApicentricError::runtime_error(format!("Event read failed: {e}"), None::<String>)
            })? {
                Event::Key(key) if key.code == KeyCode::Char('q') => break,
                Event::Key(key)
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    break
                }
                _ => {}
            }
        }
    }

    Ok(())
}
