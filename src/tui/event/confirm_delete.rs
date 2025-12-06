use crate::tui::app::{App, AppState};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(app: &mut App, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
    match key.code {
        KeyCode::Char('y') => {
            app.delete_selected_profile()?;
            app.state = AppState::List;
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.state = AppState::List;
        }
        _ => {}
    }
    Ok(())
}
