use crate::{
    GLOBAL_PROFILE_MARK,
    tui::app::{App, AppState},
};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(app: &mut App, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
    let list_component = &mut app.list_component;

    if list_component.is_searching() {
        if key
            .modifiers
            .contains(ratatui::crossterm::event::KeyModifiers::CONTROL)
        {
            match key.code {
                KeyCode::Char('d') => {
                    if !list_component.filtered_profiles().is_empty() {
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
                list_component.search_input_mut().enter_char(c);
                list_component.set_selected_index(0);
            }
            KeyCode::Backspace => {
                list_component.search_input_mut().delete_char();
                list_component.set_selected_index(0);
            }
            KeyCode::Left => {
                list_component.search_input_mut().move_cursor_left();
            }
            KeyCode::Right => {
                list_component.search_input_mut().move_cursor_right();
            }
            KeyCode::Down => {
                list_component.next();
            }
            KeyCode::Up => {
                list_component.previous();
            }
            KeyCode::Enter => {
                if let Some(name) = list_component.current_profile() {
                    let name = name.to_string();
                    app.start_editing(&name);
                }
            }
            KeyCode::F(2) => {
                if list_component.current_profile().is_some() {
                    app.state = AppState::Rename;
                    list_component.start_rename();
                }
            }
            _ => {}
        }
    } else {
        match key.code {
            KeyCode::Esc => {
                if app.list_component.unsaved_count() > 0 {
                    app.state = AppState::ConfirmExit;
                } else {
                    app.shutdown = true;
                }
            }
            KeyCode::Char('/') => {
                list_component.enter_search_mode();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                app.list_component.next();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.list_component.previous();
            }
            KeyCode::Enter => {
                if let Some(name) = list_component.current_profile() {
                    let name = name.to_string();
                    app.start_editing(&name);
                }
            }
            KeyCode::Char('s') => {
                app.save_selected()?;
            }
            KeyCode::Char('w') => {
                app.save_all()?;
            }
            KeyCode::Char('d') => {
                if let Some(name) = list_component.current_profile() {
                    if name == GLOBAL_PROFILE_MARK {
                        app.status_message = Some("Cannot delete GLOBAL profile".to_string());
                    } else {
                        app.state = AppState::ConfirmDelete;
                    }
                }
            }
            KeyCode::Char('n') => {
                app.state = AppState::AddNew;
                app.add_new_component.reset();
            }
            KeyCode::F(2) => {
                if let Some(name) = list_component.current_profile() {
                    if name == GLOBAL_PROFILE_MARK {
                        app.status_message = Some("Cannot rename GLOBAL profile".to_string());
                    } else {
                        app.state = AppState::Rename;
                        list_component.start_rename();
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}
