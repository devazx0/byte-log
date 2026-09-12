pub mod app;
pub mod events;
pub mod ui;

pub use app::{App, TimeHorizon};
pub use events::{Event, EventHandler};

use crate::error::{PcStatsError, Result};
use crossterm::{
    event::KeyCode,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::path::PathBuf;

pub fn run_tui(db_path: PathBuf, refresh_rate_ms: u64) -> Result<()> {
    enable_raw_mode().map_err(PcStatsError::Io)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(PcStatsError::Io)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(PcStatsError::Io)?;

    let mut app = App::new(db_path);
    let events = EventHandler::new(refresh_rate_ms);

    let res = run_loop(&mut terminal, &mut app, &events);

    // Always restore terminal on exit
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();

    res
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    events: &EventHandler,
) -> Result<()> {
    while app.running {
        terminal
            .draw(|f| ui::draw(f, app))
            .map_err(PcStatsError::Io)?;

        match events.next().map_err(PcStatsError::Io)? {
            Event::Tick => {
                app.refresh();
            }
            Event::Key(key) => {
                if app.category_modal.is_some() {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            app.close_category_modal();
                        }
                        KeyCode::Char(c) if ('1'..='9').contains(&c) => {
                            let idx = (c as u8 - b'1') as usize;
                            if idx < crate::tui::app::CATEGORIES.len() {
                                app.apply_category(crate::tui::app::CATEGORIES[idx]);
                            }
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.running = false;
                        }
                        KeyCode::Char('c') | KeyCode::Enter => {
                            app.open_category_modal();
                        }
                        KeyCode::Char('1') => {
                            app.set_horizon(TimeHorizon::Today);
                        }
                        KeyCode::Char('2') => {
                            app.set_horizon(TimeHorizon::Week);
                        }
                        KeyCode::Char('3') => {
                            app.set_horizon(TimeHorizon::Month);
                        }
                        KeyCode::Char('r') => {
                            app.refresh();
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            app.scroll_apps_down();
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            app.scroll_apps_up();
                        }
                        KeyCode::Tab => {
                            app.cycle_focus();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}
