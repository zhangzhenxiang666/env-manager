use crate::tui::app::{App, AppState};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(app: &mut App, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
    let list_componet = &app.list_component;
    match key.code {
        KeyCode::Esc => {
            app.shutdown = true;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.next();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.previous();
        }
        KeyCode::Enter => {
            if !list_componet.profile_names.is_empty() {
                app.state = AppState::Edit;
            }
        }
        KeyCode::Char('s') => {
            app.save_selected()?;
        }
        KeyCode::Char('w') => {
            app.save_all()?;
        }
        KeyCode::Char('d') => {
            if !list_componet.profile_names.is_empty() {
                app.state = AppState::ConfirmDelete;
            }
        }
        KeyCode::Char('n') => {
            app.state = AppState::AddNew;
            app.add_new_component.reset();
        }
        _ => {}
    }
    Ok(())
}
