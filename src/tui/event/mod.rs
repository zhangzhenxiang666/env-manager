use super::app::App;
use crate::tui::app::AppState;
use ratatui::crossterm::event::{self, Event};

mod add_new;
mod confirm_delete;
mod edit;
mod list;

pub fn handle_event(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    if let Event::Key(key) = event::read()? {
        app.status_message = None;

        if key.kind == event::KeyEventKind::Release {
            return Ok(());
        }

        match app.state {
            AppState::List => list::handle(app, key)?,
            AppState::Edit => edit::handle(app, key)?,
            AppState::ConfirmDelete => confirm_delete::handle(app, key)?,
            AppState::AddNew => add_new::handle(app, key),
            _ => {}
        }
    }
    Ok(())
}
