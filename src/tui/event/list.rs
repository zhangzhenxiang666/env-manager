use crate::tui::app::{App, AppState};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(app: &mut App, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
    let list_component = &mut app.list_component;

    if list_component.in_search_mode {
        if key
            .modifiers
            .contains(ratatui::crossterm::event::KeyModifiers::CONTROL)
        {
            match key.code {
                KeyCode::Char('d') => {
                    if !list_component.get_filtered_profiles().is_empty() {
                        app.state = AppState::ConfirmDelete;
                    }
                }
                KeyCode::Char('s') => {
                    app.save_selected()?;
                }
                KeyCode::Char('w') => {
                    app.save_all()?;
                }
                _ => {}
            }
            return Ok(());
        }

        match key.code {
            KeyCode::Esc => {
                list_component.exit_search_mode();
            }
            KeyCode::Char(c) => {
                list_component.search_input.enter_char(c);
                // Reset selection to top when search changes, or keep valid relative?
                // Usually reseting to 0 is safer as the list changes completely.
                list_component.selected_index = 0;
            }
            KeyCode::Backspace => {
                list_component.search_input.delete_char();
                list_component.selected_index = 0;
            }
            KeyCode::Left => {
                list_component.search_input.move_cursor_left();
            }
            KeyCode::Right => {
                list_component.search_input.move_cursor_right();
            }
            KeyCode::Down => {
                // Navigation within filtered list
                list_component.next();
            }
            KeyCode::Up => {
                // Navigation within filtered list
                list_component.previous();
            }
            KeyCode::Enter => {
                // Allow entering Edit mode from search mode
                if !list_component.get_filtered_profiles().is_empty() {
                    app.state = AppState::Edit;
                    // Optionally exit search mode?
                    // User didn't specify, but often you want to see context.
                    // But if we go to Edit, we are editing that specific profile.
                    // Let's keep search mode active so when they come back the filter is there?
                    // Or clearer to exit?
                    // Let's keep it for now as it's less destructive.
                }
            }
            KeyCode::F(2) => {
                let filtered = list_component.get_filtered_profiles();
                if !filtered.is_empty() {
                    let current_name = filtered[list_component.selected_index].clone();
                    app.state = AppState::Rename;
                    list_component.rename_input.text = current_name.clone();
                    list_component.rename_input.cursor_position = current_name.len();
                    list_component.rename_input.is_valid = true;
                    list_component.rename_input.error_message = None;
                }
            }
            _ => {}
        }
    } else {
        match key.code {
            KeyCode::Esc => {
                app.shutdown = true;
            }
            KeyCode::Char('/') => {
                list_component.in_search_mode = true;
                list_component.search_input.reset();
                list_component.selected_index = 0;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                app.next();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.previous();
            }
            KeyCode::Enter => {
                if !list_component.profile_names.is_empty() {
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
                if !list_component.profile_names.is_empty() {
                    app.state = AppState::ConfirmDelete;
                }
            }
            KeyCode::Char('n') => {
                app.state = AppState::AddNew;
                app.add_new_component.reset();
            }
            KeyCode::F(2) => {
                if !list_component.profile_names.is_empty() {
                    let current_name =
                        list_component.profile_names[list_component.selected_index].clone();
                    app.state = AppState::Rename;
                    list_component.rename_input.text = current_name.clone();
                    list_component.rename_input.cursor_position = current_name.len();
                    list_component.rename_input.is_valid = true;
                    list_component.rename_input.error_message = None;
                }
            }
            _ => {}
        }
    }
    Ok(())
}
