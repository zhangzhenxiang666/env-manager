use crate::tui::app::{App, AppState};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(app: &mut App, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
    if key.code == KeyCode::Esc {
        app.state = AppState::List;
    }
    Ok(())
}
