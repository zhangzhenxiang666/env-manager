use super::app::App;
use crate::tui::app::AppState;
use crate::tui::views::{add_new, edit, list};
use ratatui::crossterm::event::{self, Event};

mod confirm_delete;
mod confirm_exit;

pub fn handle_event(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    if let Event::Key(key) = event::read()? {
        app.status_message = None;

        if key.kind == event::KeyEventKind::Release {
            return Ok(());
        }

        match app.state {
            AppState::List => list::handle_event(app, key)?,
            AppState::Edit => {
                edit::handle_event(app, key);
            }
            AppState::ConfirmDelete => confirm_delete::handle(app, key)?,
            AppState::Rename => list::handle_rename_event(app, key)?,
            AppState::AddNew => {
                add_new::handle_event(app, key);
            }
            AppState::ConfirmExit => confirm_exit::handle(app, key)?,
        }
    }
    Ok(())
}
