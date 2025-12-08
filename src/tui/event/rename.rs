use crate::tui::app::{App, AppState};
use crate::tui::utils::{validate_no_spaces, validate_non_empty, validate_starts_with_non_digit};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(app: &mut App, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
    let input = app.list_component.rename_input_mut();

    match key.code {
        KeyCode::Char(c) => {
            input.enter_char(c);
            validate_input(input);
        }
        KeyCode::Backspace => {
            input.delete_char();
            validate_input(input);
        }
        KeyCode::Left => {
            input.move_cursor_left();
        }
        KeyCode::Right => {
            input.move_cursor_right();
        }
        KeyCode::Esc => {
            app.list_component.reset_rename();
            app.state = AppState::List;
        }
        KeyCode::Enter => {
            if input.is_valid {
                let new_name = input.text.clone();
                app.rename_profile(new_name)?;
                app.list_component.reset_rename();
                app.state = AppState::List;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_input(input: &mut crate::tui::utils::Input) {
    input.is_valid = true;
    input.error_message = None;

    if let Err(e) = validate_non_empty(&input.text) {
        input.set_error_message(&e);
        return;
    }
    if let Err(e) = validate_no_spaces(&input.text) {
        input.set_error_message(&e);
        return;
    }
    if let Err(e) = validate_starts_with_non_digit(&input.text) {
        input.set_error_message(&e);
    }
}
