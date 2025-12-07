use crate::tui::{
    app::{App, AppState},
    components::edit::{EditComponent, EditFocus, EditVariableFocus},
};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(app: &mut App, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
    if app.edit_component.show_select_popup {
        app.edit_component.on_popup_input(key);
        return Ok(());
    }

    if app.edit_component.is_editing_variable {
        handle_editing_mode(app, key);
    } else {
        handle_navigation_mode(app, key);
    }
    Ok(())
}

fn handle_editing_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => handle_editing_enter(app),
        KeyCode::Tab => handle_editing_tab(app),
        KeyCode::BackTab => handle_editing_tab(app),
        KeyCode::Esc => handle_editing_esc(app),
        _ => handle_editing_input(app, key.code),
    }
}

fn handle_editing_enter(app: &mut App) {
    let edit = &mut app.edit_component;

    // Validate before confirming if editing Key
    if edit.variable_column_focus == EditVariableFocus::Key && !validate_variable_key_input(edit) {
        return;
    }

    edit.confirm_editing_variable();

    if edit.variable_column_focus == EditVariableFocus::Key {
        edit.switch_variable_column();
        edit.start_editing_variable();
    }
}

fn handle_editing_tab(app: &mut App) {
    let edit = &mut app.edit_component;

    // Validate before switching if currently on Key
    if edit.variable_column_focus == EditVariableFocus::Key && !validate_variable_key_input(edit) {
        return;
    }

    edit.confirm_editing_variable();
    edit.switch_variable_column();
    edit.start_editing_variable();
}

fn handle_editing_esc(app: &mut App) {
    let edit = &mut app.edit_component;
    edit.cancel_editing_variable();

    // Check if the current row is invalid and delete if so
    if should_delete_variable_row(edit) {
        edit.delete_variable();
    }
}

fn handle_editing_input(app: &mut App, key_code: KeyCode) {
    let edit = &mut app.edit_component;
    match key_code {
        KeyCode::Char(c) => {
            if let Some(input) = edit.get_focused_variable_input_mut() {
                input.enter_char(c);
            }
        }
        KeyCode::Backspace => {
            if let Some(input) = edit.get_focused_variable_input_mut() {
                input.delete_char();
            }
        }
        KeyCode::Left => {
            if let Some(input) = edit.get_focused_variable_input_mut() {
                input.move_cursor_left();
            }
        }
        KeyCode::Right => {
            if let Some(input) = edit.get_focused_variable_input_mut() {
                input.move_cursor_right();
            }
        }
        _ => edit.confirm_editing_variable(),
    }
}

fn handle_navigation_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            update_profile_from_edit(app);
            app.state = AppState::List;
        }
        KeyCode::Tab => app.edit_component.switch_focus(),

        // Navigation within section
        KeyCode::Up | KeyCode::Char('k') => match app.edit_component.focus {
            EditFocus::Variables => app.edit_component.select_previous_variable(),
            EditFocus::Profiles => app.edit_component.select_previous_profile(),
        },
        KeyCode::Down | KeyCode::Char('j') => match app.edit_component.focus {
            EditFocus::Variables => app.edit_component.select_next_variable(),
            EditFocus::Profiles => app.edit_component.select_next_profile(),
        },

        // Variables actions
        KeyCode::Char('a') if app.edit_component.focus == EditFocus::Variables => {
            app.edit_component.add_variable();
        }
        KeyCode::Char('d') => match app.edit_component.focus {
            EditFocus::Variables => app.edit_component.delete_variable(),
            EditFocus::Profiles => app.edit_component.remove_profile_dependency(),
        },
        KeyCode::Char('e') if app.edit_component.focus == EditFocus::Variables => {
            app.edit_component.start_editing_variable();
        }
        KeyCode::Left | KeyCode::Char('h') if app.edit_component.focus == EditFocus::Variables => {
            app.edit_component.switch_variable_column();
        }
        KeyCode::Right | KeyCode::Char('l') if app.edit_component.focus == EditFocus::Variables => {
            app.edit_component.switch_variable_column();
        }

        // Profiles actions
        KeyCode::Char('n') if app.edit_component.focus == EditFocus::Profiles => {
            let current_name = app.edit_component.profile_name.clone();
            // We need to calculate candidates from app graph
            // Candidates are: All profiles EXCEPT:
            // 1. Myself
            // 2. Already dependencies
            // 3. Profiles that would cause a cycle

            let mut candidates = Vec::new();
            let graph = &app.config_manager.app_config.graph;

            // We need to separate this because we can't borrow app.edit_component and app.config_manager at same time if one is mut?
            // Actually app.edit_component is a field. app.config_manager is another field.
            // But we are borrowing `app` as mut.
            // Split borrows is hard with &mut App.
            // We can iterate keys first.
            let profile_keys: Vec<String> = app
                .config_manager
                .app_config
                .profiles
                .keys()
                .cloned()
                .collect();

            for candidate in profile_keys {
                if candidate == current_name {
                    continue;
                }
                // Check if already dependency
                if app.edit_component.profiles.contains(&candidate) {
                    continue;
                }

                // Cycle detection
                let mut creates_cycle = false;
                if let (Some(&start), Some(&end)) = (
                    graph.profile_nodes.get(&candidate),
                    graph.profile_nodes.get(&current_name),
                ) {
                    if daggy::petgraph::algo::has_path_connecting(&graph.graph, start, end, None) {
                        creates_cycle = true;
                    }
                }

                if !creates_cycle {
                    candidates.push(candidate);
                }
            }
            // Sort
            candidates.sort();

            // Now open popup
            app.edit_component.open_add_dependency_popup(candidates);
        }

        _ => {}
    }
}

fn validate_variable_key_input(edit: &mut EditComponent) -> bool {
    use crate::tui::utils;

    if let Some(input) = edit.get_focused_variable_input_mut() {
        if let Err(e) = utils::validate_non_empty(&input.text) {
            input.set_error_message(&e);
            return false;
        }
        if let Err(e) = utils::validate_no_spaces(&input.text) {
            input.set_error_message(&e);
            return false;
        }
        if let Err(e) = utils::validate_starts_with_non_digit(&input.text) {
            input.set_error_message(&e);
            return false;
        }
        input.is_valid = true;
        input.error_message = None;
        return true;
    }
    true
}

fn should_delete_variable_row(edit: &EditComponent) -> bool {
    use crate::tui::utils;

    if let Some((key_input, _)) = edit.variables.get(edit.selected_variable_index) {
        utils::validate_non_empty(&key_input.text).is_err()
            || utils::validate_no_spaces(&key_input.text).is_err()
            || utils::validate_starts_with_non_digit(&key_input.text).is_err()
    } else {
        false
    }
}

fn update_profile_from_edit(app: &mut App) {
    let new_profile = app.edit_component.to_profile();
    let name = &app.edit_component.profile_name;

    app.config_manager
        .app_config
        .profiles
        .insert(name.clone(), new_profile);

    app.list_component.dirty_profiles.insert(name.clone());

    match crate::config::graph::ProfileGraph::build(&app.config_manager.app_config.profiles) {
        Ok(graph) => {
            app.config_manager.app_config.graph = graph;
            app.status_message = Some(format!("Updated '{name}' in memory."));
        }
        Err(e) => {
            app.status_message = Some(format!("Warning: Graph invalid: {e}"));
        }
    }
}
