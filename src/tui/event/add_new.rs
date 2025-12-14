use crate::GLOBAL_PROFILE_MARK;
use crate::config::models::Profile;
use crate::tui::app::{App, AppState};
use crate::tui::components::add_new::{AddNewComponent, AddNewFocus, AddNewVariableFocus};
use crate::tui::utils::validate_input;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

pub fn handle(app: &mut App, key: KeyEvent) {
    if app.add_new_component.is_editing() {
        handle_editing_mode(app, key);
    } else {
        handle_navigation_mode(app, key);
    }
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
    let add_new = &mut app.add_new_component;

    // Validate before confirming if editing Key
    if add_new.variable_column_focus() == AddNewVariableFocus::Key
        && !validate_variable_key_input(add_new)
    {
        return;
    }

    add_new.confirm_editing_variable();

    if add_new.variable_column_focus() == AddNewVariableFocus::Key {
        add_new.switch_variable_column();
        add_new.start_editing_variable();
    }
}

fn handle_editing_tab(app: &mut App) {
    let add_new = &mut app.add_new_component;

    // Validate before switching if currently on Key
    if add_new.variable_column_focus() == AddNewVariableFocus::Key
        && !validate_variable_key_input(add_new)
    {
        return;
    }

    add_new.confirm_editing_variable();
    add_new.switch_variable_column();
    add_new.start_editing_variable();
}

fn handle_editing_esc(app: &mut App) {
    let add_new = &mut app.add_new_component;
    add_new.cancel_editing_variable();

    // Check if the current row is invalid (empty, spaces, etc.) and delete if so
    if should_delete_variable_row(add_new) {
        add_new.delete_selected_variable();
    }
}

fn handle_editing_input(app: &mut App, key_code: KeyCode) {
    let add_new = &mut app.add_new_component;
    match key_code {
        KeyCode::Char(c) => {
            if let Some(input) = add_new.get_focused_variable_input_mut() {
                input.enter_char(c);

                if add_new.variable_column_focus() == AddNewVariableFocus::Key {
                    validate_variable_key_input(add_new);
                }
            }
        }
        KeyCode::Backspace => {
            if let Some(input) = add_new.get_focused_variable_input_mut() {
                input.delete_char();

                if add_new.variable_column_focus() == AddNewVariableFocus::Key {
                    validate_variable_key_input(add_new);
                }
            }
        }
        KeyCode::Left => {
            if let Some(input) = add_new.get_focused_variable_input_mut() {
                input.move_cursor_left();
            }
        }
        KeyCode::Right => {
            if let Some(input) = add_new.get_focused_variable_input_mut() {
                input.move_cursor_right();
            }
        }
        // For any other key, confirm the current edit
        _ => {
            if validate_variable_key_input(add_new) {
                add_new.confirm_editing_variable();
            }
        }
    }
}

fn handle_navigation_mode(app: &mut App, key: KeyEvent) {
    match key {
        // Save
        KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => save_profile(app),

        // Close / Cancel
        KeyEvent {
            code: KeyCode::Esc, ..
        } => close_popup(app),

        // Navigation
        KeyEvent {
            code: KeyCode::Tab, ..
        } => attempt_switch_focus(app, true),

        KeyEvent {
            code: KeyCode::BackTab,
            ..
        } => attempt_switch_focus(app, false),

        // Context Specific
        _ => dispatch_context_key(app, key),
    }
}

fn save_profile(app: &mut App) {
    if !validate_name(app) {
        return;
    }

    let add_new = &mut app.add_new_component;
    let new_name = add_new.name_input().text().trim().to_string();

    let variables_map: HashMap<String, String> = add_new
        .variables_for_rendering()
        .iter()
        .map(|(k, v)| (k.text().to_string(), v.text().to_string()))
        .filter(|(k, _)| !k.is_empty())
        .collect();

    let new_profile = Profile {
        profiles: add_new.added_profiles().iter().cloned().collect(),
        variables: variables_map,
    };

    // 1. Add profile to memory
    app.config_manager
        .add_profile(new_name.clone(), new_profile.clone());
    app.list_component.mark_dirty(new_name.clone());

    // 2. Add node to graph
    app.config_manager.add_profile_node(new_name.clone());

    // 3. Add dependency edges to graph
    for dep_name in &new_profile.profiles {
        if let Err(e) = app.config_manager.add_dependency_edge(&new_name, dep_name) {
            app.status_message = Some(format!(
                "Warning: Failed to add dependency edge to '{dep_name}': {e}"
            ));
        }
    }

    // 4. Update UI list
    let mut profiles = app.list_component.all_profiles().to_vec();
    profiles.push(new_name.clone());
    profiles.sort();
    app.list_component.update_profiles(profiles);

    if let Some(index) = app
        .list_component
        .all_profiles()
        .iter()
        .position(|r| r == &new_name)
    {
        app.list_component.set_selected_index(index);
    }

    app.status_message = Some(format!("Profile '{new_name}' created."));
    app.state = AppState::List;
    add_new.reset();
}

fn close_popup(app: &mut App) {
    app.state = AppState::List;
    app.add_new_component.reset();
}

fn attempt_switch_focus(app: &mut App, forward: bool) {
    // If focused on Name, validate before leaving
    if app.add_new_component.current_focus() == AddNewFocus::Name && !validate_name(app) {
        return;
    }
    app.add_new_component.switch_focus(forward);
}

fn dispatch_context_key(app: &mut App, key: KeyEvent) {
    let focus = app.add_new_component.current_focus();

    match key.code {
        KeyCode::Esc => {
            app.add_new_component.reset();
            app.state = AppState::List;
        }
        KeyCode::Char(c) if focus == AddNewFocus::Name => {
            app.add_new_component.name_input_mut().enter_char(c);
            validate_name(app);
        }
        KeyCode::Backspace if focus == AddNewFocus::Name => {
            app.add_new_component.name_input_mut().delete_char();
            validate_name(app);
        }
        KeyCode::Left if focus == AddNewFocus::Name => {
            app.add_new_component.name_input_mut().move_cursor_left()
        }
        KeyCode::Right if focus == AddNewFocus::Name => {
            app.add_new_component.name_input_mut().move_cursor_right()
        }
        KeyCode::Enter if focus == AddNewFocus::Name && validate_name(app) => {
            app.add_new_component.switch_focus(true);
        }
        _ => {
            // Dispatch to specific handlers for Profiles and Variables
            match focus {
                AddNewFocus::Profiles => profiles(app, key.code),
                AddNewFocus::Variables => variables(app, key.code),
                _ => {}
            }
        }
    }
}

fn profiles(app: &mut App, key_code: KeyCode) {
    let add_new = &mut app.add_new_component;
    let available_profiles: Vec<_> = app
        .list_component
        .all_profiles()
        .iter()
        .filter(|name| **name != add_new.name_input().text() && *name != GLOBAL_PROFILE_MARK)
        .collect();
    let count = available_profiles.len();

    match key_code {
        KeyCode::Up | KeyCode::Char('k') => add_new.select_previous_profile(count),
        KeyCode::Down | KeyCode::Char('j') => add_new.select_next_profile(count),
        KeyCode::Enter | KeyCode::Char(' ') => {
            if let Some(selected_name) = available_profiles.get(add_new.profiles_selection_index())
            {
                add_new.toggle_current_profile(selected_name.to_string());
            }
        }
        _ => {}
    }
}

fn variables(app: &mut App, key_code: KeyCode) {
    let add_new = &mut app.add_new_component;
    match key_code {
        KeyCode::Up | KeyCode::Char('k') => add_new.select_previous_variable(),
        KeyCode::Down | KeyCode::Char('j') => add_new.select_next_variable(),
        KeyCode::Left | KeyCode::Char('h') => add_new.switch_variable_column(),
        KeyCode::Right | KeyCode::Char('l') => add_new.switch_variable_column(),
        KeyCode::Char('a') => add_new.add_new_variable(),
        KeyCode::Char('d') => add_new.delete_selected_variable(),
        KeyCode::Char('e') => add_new.start_editing_variable(),
        _ => {}
    }
}

fn validate_name(app: &mut App) -> bool {
    let input = app.add_new_component.name_input_mut();
    input.clear_error();
    if app.config_manager.has_profile(input.text()) {
        input.set_error_message("Profile already exists");
        false
    } else {
        validate_input(input)
    }
}

/// Validates the currently focused variable input (if it's a Key).
/// Returns true if valid, false if invalid.
fn validate_variable_key_input(add_new: &mut AddNewComponent) -> bool {
    if let Some(input) = add_new.get_focused_variable_input_mut() {
        input.clear_error();
        validate_input(input)
    } else {
        true
    }
}

fn should_delete_variable_row(add_new: &AddNewComponent) -> bool {
    let idx = add_new.selected_variable_index();
    !add_new.is_variable_valid(idx)
}
