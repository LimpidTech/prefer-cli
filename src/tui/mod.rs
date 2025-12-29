mod editing;
mod input;
mod navigation;
mod render;
mod state;
mod tree;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::{Path, PathBuf};

use crate::backend::ConfigBackend;
use crate::settings::Settings;
use input::{execute_command, handle_input};
use render::ui;
use state::{App, UiState};

pub fn run(file: &Path, backend: &dyn ConfigBackend) -> Result<()> {
    let settings = Settings::load();
    let config = backend.load(file)?;
    let info = backend.info(file)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend_term = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend_term)?;

    let mut app = App::new(
        config,
        info.path.clone(),
        PathBuf::from(&info.path),
        settings.mode,
        backend,
    );
    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            app.message = None;

            if app.ui_state == UiState::Command && key.code == KeyCode::Enter {
                if execute_command(app) {
                    return Ok(());
                }
                continue;
            }

            if handle_input(app, key.code, key.modifiers) {
                return Ok(());
            }
        }
    }
}
