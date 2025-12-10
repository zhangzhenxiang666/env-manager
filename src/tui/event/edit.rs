use crate::GLOBAL_PROFILE_MARK;
use crate::tui::app::{App, AppState};
use crate::tui::components::edit::{EditComponent, EditFocus, EditVariableFocus};
use crate::tui::utils::{validate_no_spaces, validate_non_empty, validate_starts_with_non_digit};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(app: &mut App, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
    if app.edit_component.is_dependency_selector_open() {
        handle_dependency_selector(app, key);
        return Ok(());
    }

    if app.edit_component.is_editing() {
        handle_variable_editing_mode(app, key);
    } else {
        handle_navigation_mode(app, key);
    }
    Ok(())
}

fn handle_dependency_selector(app: &mut App, key: KeyEvent) {
    if let Some(selected_deps) = app.edit_component.handle_selector_input(key) {
        add_dependencies_to_profile(app, selected_deps);
    }
}

fn add_dependencies_to_profile(app: &mut App, dep_names: Vec<String>) {
    let profile_name = app.edit_component.profile_name().to_string();
    if profile_name == GLOBAL_PROFILE_MARK {
        dep_names
            .into_iter()
            .for_each(|name| app.edit_component.add_profile_dependency(name));
    } else {
        for dep_name in dep_names {
            // Try to add to graph first (validation)
            match app
                .config_manager
                .add_dependency_edge(&profile_name, &dep_name)
            {
                Ok(_) => {
                    // Success: update UI component
                    app.edit_component.add_profile_dependency(dep_name);
                }
                Err(e) => {
                    // Failed: show error, don't update UI
                    app.status_message = Some(format!("Cannot add dependency '{dep_name}': {e}"));
                }
            }
        }
    }

    mark_profile_as_dirty_if_changed(app);
}

fn remove_dependency_from_profile(app: &mut App) {
    let profile_name = app.edit_component.profile_name().to_string();
    let selected_idx = app.edit_component.selected_profile_index();
    if profile_name == GLOBAL_PROFILE_MARK {
        app.edit_component.remove_profile_dependency();
    } else if let Some(removed_dep) = app.edit_component.profiles().get(selected_idx) {
        let removed_dep = removed_dep.clone();

        // Update UI component
        app.edit_component.remove_profile_dependency();

        // Update graph immediately (incremental)
        if let Err(e) = app
            .config_manager
            .remove_dependency_edge(&profile_name, &removed_dep)
        {
            app.status_message = Some(format!("Failed to remove dependency: {e}"));
        }
    }

    mark_profile_as_dirty_if_changed(app);
}

fn open_dependency_selector(app: &mut App) {
    let current_profile = app.edit_component.profile_name();
    let existing_deps = app.edit_component.profiles();

    // Get profiles that depend on current (would create cycle)
    let ancestors: std::collections::HashSet<String> = app
        .config_manager
        .get_parents(current_profile)
        .unwrap_or_default()
        .into_iter()
        .collect();

    // Filter available profiles
    let available: Vec<String> = app
        .list_component
        .all_profiles()
        .iter()
        .filter(|p| {
            let name = p.as_str();
            name != current_profile           // Exclude self
                && !existing_deps.contains(p)  // Exclude already added
                && !ancestors.contains(*p) // Exclude would-be-circular
                && *p != GLOBAL_PROFILE_MARK // Exclude global
        })
        .cloned()
        .collect();

    app.edit_component.open_dependency_selector(available);
}

fn handle_variable_editing_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => confirm_and_maybe_switch_column(app),
        KeyCode::Tab | KeyCode::BackTab => confirm_and_switch_column(app),
        KeyCode::Esc => cancel_variable_editing(app),
        _ => handle_text_input(app, key.code),
    }
}

fn confirm_and_maybe_switch_column(app: &mut App) {
    let should_switch = {
        let edit = &mut app.edit_component;

        // Validate Key before confirming
        if edit.variable_column_focus() == EditVariableFocus::Key && !validate_variable_key(edit) {
            return;
        }

        edit.confirm_editing_variable();
        edit.variable_column_focus() == EditVariableFocus::Key
    };

    mark_profile_as_dirty_if_changed(app);

    if should_switch {
        app.edit_component.switch_variable_column();
        app.edit_component.start_editing_variable();
    }
}

fn confirm_and_switch_column(app: &mut App) {
    {
        let edit = &mut app.edit_component;

        // Validate Key before switching
        if edit.variable_column_focus() == EditVariableFocus::Key && !validate_variable_key(edit) {
            return;
        }

        edit.confirm_editing_variable();
    }

    mark_profile_as_dirty_if_changed(app);

    app.edit_component.switch_variable_column();
    app.edit_component.start_editing_variable();
}

fn cancel_variable_editing(app: &mut App) {
    let edit = &mut app.edit_component;
    edit.cancel_editing_variable();

    // Delete row if invalid (empty key, etc.)
    if should_delete_invalid_variable(edit) {
        edit.delete_variable();
    }
}

fn handle_text_input(app: &mut App, key_code: KeyCode) {
    let edit = &mut app.edit_component;

    if let Some(input) = edit.get_focused_variable_input_mut() {
        match key_code {
            KeyCode::Char(c) => {
                input.enter_char(c);

                if edit.variable_column_focus() == EditVariableFocus::Key {
                    validate_variable_key(edit);
                }
            }
            KeyCode::Backspace => {
                input.delete_char();

                if edit.variable_column_focus() == EditVariableFocus::Key {
                    validate_variable_key(edit);
                }
            }
            KeyCode::Left => input.move_cursor_left(),
            KeyCode::Right => input.move_cursor_right(),
            _ => edit.confirm_editing_variable(),
        }
    }
}

fn handle_navigation_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => exit_edit_mode(app),
        KeyCode::Tab => app.edit_component.switch_focus(),

        // Navigation
        KeyCode::Char('j') | KeyCode::Down => navigate_down(app),
        KeyCode::Char('k') | KeyCode::Up => navigate_up(app),
        KeyCode::Left | KeyCode::Right => switch_column_if_in_variables(app),

        // Actions
        KeyCode::Char('a') => add_variable_if_in_variables(app),
        KeyCode::Char('d') => delete_current_item(app),
        KeyCode::Char('e') => start_editing_variable_if_in_variables(app),
        KeyCode::Char('n') => open_dependency_selector_if_in_profiles(app),

        _ => {}
    }
}

fn exit_edit_mode(app: &mut App) {
    // Save profile if there are changes
    if app.edit_component.has_changes() {
        save_profile_to_memory(app);
    }
    app.state = AppState::List;
}

fn navigate_down(app: &mut App) {
    match app.edit_component.current_focus() {
        EditFocus::Variables => app.edit_component.select_next_variable(),
        EditFocus::Profiles => app.edit_component.select_next_profile(),
    }
}

fn navigate_up(app: &mut App) {
    match app.edit_component.current_focus() {
        EditFocus::Variables => app.edit_component.select_previous_variable(),
        EditFocus::Profiles => app.edit_component.select_previous_profile(),
    }
}

fn switch_column_if_in_variables(app: &mut App) {
    if app.edit_component.current_focus() == EditFocus::Variables {
        app.edit_component.switch_variable_column();
    }
}

fn add_variable_if_in_variables(app: &mut App) {
    if app.edit_component.current_focus() == EditFocus::Variables {
        app.edit_component.add_variable();
        mark_profile_as_dirty_if_changed(app);
    }
}

fn delete_current_item(app: &mut App) {
    match app.edit_component.current_focus() {
        EditFocus::Variables => {
            app.edit_component.delete_variable();
            mark_profile_as_dirty_if_changed(app);
        }
        EditFocus::Profiles => {
            remove_dependency_from_profile(app);
        }
    }
}

fn start_editing_variable_if_in_variables(app: &mut App) {
    if app.edit_component.current_focus() == EditFocus::Variables {
        app.edit_component.start_editing_variable();
    }
}

fn open_dependency_selector_if_in_profiles(app: &mut App) {
    if app.edit_component.current_focus() == EditFocus::Profiles {
        open_dependency_selector(app);
    }
}

/// Validate variable key (non-empty, no spaces, not start with digit)
fn validate_variable_key(edit: &mut EditComponent) -> bool {
    if let Some(input) = edit.get_focused_variable_input_mut() {
        input.clear_error();

        if let Err(e) = validate_non_empty(input.text()) {
            input.set_error_message(&e);
            return false;
        }
        if let Err(e) = validate_no_spaces(input.text()) {
            input.set_error_message(&e);
            return false;
        }
        if let Err(e) = validate_starts_with_non_digit(input.text()) {
            input.set_error_message(&e);
            return false;
        }
        true
    } else {
        false
    }
}

/// Check if current variable row is invalid and should be deleted
fn should_delete_invalid_variable(edit: &EditComponent) -> bool {
    let idx = edit.selected_variable_index();
    !edit.is_variable_valid(idx)
}

/// Save edited profile to memory (called on Esc)
fn save_profile_to_memory(app: &mut App) {
    let name = app.edit_component.profile_name().to_string();
    let new_profile = app.edit_component.to_profile();

    // Update profile in memory
    // Update profile in memory
    app.config_manager
        .add_profile(name.clone(), new_profile.clone());

    if name == GLOBAL_PROFILE_MARK {
        if let Err(e) = app.config_manager.write_global(&new_profile) {
            app.status_message = Some(format!("Error saving GLOBAL: {}", e));
        } else {
            app.list_component.clear_dirty(&name);
        }
    } else {
        app.list_component.mark_dirty(name);
    }
}

/// Mark profile as dirty if there are any changes
fn mark_profile_as_dirty_if_changed(app: &mut App) {
    if app.edit_component.has_changes() {
        let name = app.edit_component.profile_name().to_string();
        app.list_component.mark_dirty(name);
    }
}
