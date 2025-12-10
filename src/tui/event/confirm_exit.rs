use crate::tui::app::{App, AppState};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(app: &mut App, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => {
            app.save_all()?;
            app.shutdown = true;
        }
        KeyCode::Char('n') => {
            app.shutdown = true;
        }
        KeyCode::Esc => {
            app.state = AppState::List;
        }
        _ => {}
    }
    Ok(())
}
