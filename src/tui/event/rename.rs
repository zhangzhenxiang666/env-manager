use crate::tui::app::{App, AppState};
use crate::tui::utils::validate_input;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(app: &mut App, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
    match key.code {
        KeyCode::Char(c) => {
            app.list_component.rename_input_mut().enter_char(c);
            validate_name(app);
        }
        KeyCode::Backspace => {
            app.list_component.rename_input_mut().delete_char();
            validate_name(app);
        }
        KeyCode::Left => {
            app.list_component.rename_input_mut().move_cursor_left();
        }
        KeyCode::Right => {
            app.list_component.rename_input_mut().move_cursor_right();
        }
        KeyCode::Esc => {
            app.list_component.reset_rename();
            app.state = AppState::List;
        }
        KeyCode::Enter => {
            if app.list_component.rename_input_mut().is_valid() {
                let new_name = app.list_component.rename_input().text().to_string();
                app.rename_profile(new_name)?;
                app.list_component.reset_rename();
                app.state = AppState::List;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_name(app: &mut App) {
    app.list_component.rename_input_mut().clear_error();

    if let Some(name) = app.list_component.current_profile()
        && name != app.list_component.rename_input().text()
        && app
            .config_manager
            .has_profile(app.list_component.rename_input().text())
    {
        app.list_component
            .rename_input_mut()
            .set_error_message("Profile name already exists");
        return;
    }
    validate_input(app.list_component.rename_input_mut());
}
