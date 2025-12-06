use crate::{
    config::models::Profile,
    tui::{
        app::{App, AppState},
        components::add_new::{AddNewComponent, AddNewFocus, AddNewVariableFocus},
        utils,
    },
};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

pub fn handle(app: &mut App, key: KeyEvent) {
    if app.add_new_component.is_editing_variable {
        handle_editing_mode(app, key);
    } else {
        handle_navigation_mode(app, key);
    }
}

fn handle_editing_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => handle_editing_enter(app),
        KeyCode::Tab => handle_editing_tab(app),
        KeyCode::BackTab => handle_editing_tab(app), // BackTab behaves same as Tab for 2 columns
        KeyCode::Esc => handle_editing_esc(app),
        _ => handle_editing_input(app, key.code),
    }
}

fn handle_editing_enter(app: &mut App) {
    let add_new = &mut app.add_new_component;

    // Validate before confirming if editing Key
    if add_new.focused_column == AddNewVariableFocus::Key && !validate_variable_key_input(add_new) {
        return;
    }

    add_new.confirm_editing_variable();

    if add_new.focused_column == AddNewVariableFocus::Key {
        add_new.switch_variable_column();
        add_new.start_editing_variable();
    }
}

fn handle_editing_tab(app: &mut App) {
    let add_new = &mut app.add_new_component;

    // Validate before switching if currently on Key
    if add_new.focused_column == AddNewVariableFocus::Key && !validate_variable_key_input(add_new) {
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
            }
        }
        KeyCode::Backspace => {
            if let Some(input) = add_new.get_focused_variable_input_mut() {
                input.delete_char();
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
        _ => add_new.confirm_editing_variable(),
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
        _ => dispatch_context_key(app, key.code),
    }
}

fn save_profile(app: &mut App) {
    validate_name(app);
    if !app.add_new_component.name_input.is_valid {
        return;
    }

    let add_new = &mut app.add_new_component;
    let new_name = add_new.name_input.text.trim().to_string();

    let mut new_profile = Profile {
        profiles: add_new.added_profiles.iter().cloned().collect(),
        ..Default::default()
    };

    let variables_map: HashMap<String, String> = add_new
        .variables
        .iter()
        .map(|(k, v)| (k.text.clone(), v.text.clone()))
        .filter(|(k, _)| !k.is_empty())
        .collect();
    new_profile.variables = variables_map;

    app.config_manager
        .app_config
        .profiles
        .insert(new_name.clone(), new_profile);
    app.list_component.dirty_profiles.insert(new_name.clone());

    app.list_component.profile_names.push(new_name.clone());
    app.list_component.profile_names.sort();
    if let Some(index) = app
        .list_component
        .profile_names
        .iter()
        .position(|r| r == &new_name)
    {
        app.list_component.selected_index = index;
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
    if app.add_new_component.focus == AddNewFocus::Name {
        validate_name(app);
        if !app.add_new_component.name_input.is_valid {
            return;
        }
    }
    app.add_new_component.switch_focus(forward);
}

fn dispatch_context_key(app: &mut App, key_code: KeyCode) {
    let focus = app.add_new_component.focus;
    match focus {
        AddNewFocus::Name => name(app, key_code),
        AddNewFocus::Profiles => profiles(app, key_code),
        AddNewFocus::Variables => variables(app, key_code),
    }
}

fn name(app: &mut App, key_code: KeyCode) {
    match key_code {
        KeyCode::Char(c) => {
            app.add_new_component.name_input.enter_char(c);
            validate_name(app);
        }
        KeyCode::Backspace => {
            app.add_new_component.name_input.delete_char();
            validate_name(app);
        }
        KeyCode::Left => app.add_new_component.name_input.move_cursor_left(),
        KeyCode::Right => app.add_new_component.name_input.move_cursor_right(),
        KeyCode::Enter => {
            validate_name(app);
            if app.add_new_component.name_input.is_valid {
                app.add_new_component.switch_focus(true);
            }
        }
        _ => {}
    }
}

fn profiles(app: &mut App, key_code: KeyCode) {
    let add_new = &mut app.add_new_component;
    let available_profiles: Vec<_> = app
        .list_component
        .profile_names
        .iter()
        .filter(|name| **name != add_new.name_input.text)
        .collect();
    let count = available_profiles.len();

    match key_code {
        KeyCode::Up | KeyCode::Char('k') => add_new.select_previous_profile(count),
        KeyCode::Down | KeyCode::Char('j') => add_new.select_next_profile(count),
        KeyCode::Enter | KeyCode::Char(' ') => {
            if let Some(selected_name) = available_profiles.get(add_new.profiles_selection_index) {
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

// --- Validators ---

fn validate_name(app: &mut App) {
    let add_new = &mut app.add_new_component;
    let new_name = add_new.name_input.text.trim();

    if let Err(e) = utils::validate_non_empty(new_name) {
        add_new.name_input.set_error_message(&format!("Name {}", e));
        return;
    }
    if let Err(e) = utils::validate_no_spaces(new_name) {
        add_new.name_input.set_error_message(&format!("Name {}", e));
        return;
    }
    if let Err(e) = utils::validate_starts_with_non_digit(new_name) {
        add_new.name_input.set_error_message(&format!("Name {}", e));
        return;
    }

    if app
        .config_manager
        .app_config
        .profiles
        .contains_key(new_name)
    {
        add_new
            .name_input
            .set_error_message("Profile already exists");
        return;
    }
    add_new.name_input.is_valid = true;
    add_new.name_input.error_message = None;
}

/// Validates the currently focused variable input (if it's a Key).
/// Returns true if valid, false if invalid.
fn validate_variable_key_input(add_new: &mut AddNewComponent) -> bool {
    if let Some(input) = add_new.get_focused_variable_input_mut() {
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

fn should_delete_variable_row(add_new: &AddNewComponent) -> bool {
    if let Some((key_input, _)) = add_new.variables.get(add_new.selected_variable_index) {
        utils::validate_non_empty(&key_input.text).is_err()
            || utils::validate_no_spaces(&key_input.text).is_err()
            || utils::validate_starts_with_non_digit(&key_input.text).is_err()
    } else {
        false
    }
}
