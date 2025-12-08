use crate::tui::{
    app::{App, AppState},
    components::edit::{EditComponent, EditFocus, EditVariableFocus},
    utils::{validate_no_spaces, validate_non_empty, validate_starts_with_non_digit},
};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(app: &mut App, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
    // Handle dependency selector if open
    if app.edit_component.is_dependency_selector_open() {
        if let Some(selected) = app.edit_component.handle_selector_input(key) {
            // Add selected dependencies
            for profile_name in selected {
                app.edit_component.add_profile_dependency(profile_name);
            }
            // Mark as dirty if changes detected
            mark_if_changed(app);
        }
        return Ok(());
    }

    // Handle variable editing mode
    if app.edit_component.is_editing() {
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
    let should_switch = {
        let edit = &mut app.edit_component;

        // Validate before confirming if editing Key
        if edit.variable_column_focus() == EditVariableFocus::Key
            && !validate_variable_key_input(edit)
        {
            return;
        }

        edit.confirm_editing_variable();
        edit.variable_column_focus() == EditVariableFocus::Key
    };

    // Mark as dirty if changes detected
    mark_if_changed(app);

    if should_switch {
        app.edit_component.switch_variable_column();
        app.edit_component.start_editing_variable();
    }
}

fn handle_editing_tab(app: &mut App) {
    {
        let edit = &mut app.edit_component;

        // Validate before switching if currently on Key
        if edit.variable_column_focus() == EditVariableFocus::Key
            && !validate_variable_key_input(edit)
        {
            return;
        }

        edit.confirm_editing_variable();
    }

    // Mark as dirty if changes detected
    mark_if_changed(app);

    app.edit_component.switch_variable_column();
    app.edit_component.start_editing_variable();
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
        _ => {}
    }
}

fn handle_navigation_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            // Only update and mark dirty if there are actual changes
            if app.edit_component.has_changes() {
                update_profile_from_edit(app);
            }
            app.state = AppState::List;
        }
        KeyCode::Tab => {
            app.edit_component.switch_focus();
        }
        KeyCode::Char('j') | KeyCode::Down => match app.edit_component.current_focus() {
            EditFocus::Variables => app.edit_component.select_next_variable(),
            EditFocus::Profiles => app.edit_component.select_next_profile(),
        },
        KeyCode::Char('k') | KeyCode::Up => match app.edit_component.current_focus() {
            EditFocus::Variables => app.edit_component.select_previous_variable(),
            EditFocus::Profiles => app.edit_component.select_previous_profile(),
        },
        KeyCode::Left | KeyCode::Right => {
            // Switch between Key and Value columns in Variables section
            if app.edit_component.current_focus() == EditFocus::Variables {
                app.edit_component.switch_variable_column();
            }
        }
        KeyCode::Char('a') => {
            if app.edit_component.current_focus() == EditFocus::Variables {
                app.edit_component.add_variable();
                mark_if_changed(app);
            }
        }
        KeyCode::Char('d') => match app.edit_component.current_focus() {
            EditFocus::Variables => {
                app.edit_component.delete_variable();
                mark_if_changed(app);
            }
            EditFocus::Profiles => {
                app.edit_component.remove_profile_dependency();
                mark_if_changed(app);
            }
        },
        KeyCode::Char('e') => {
            if app.edit_component.current_focus() == EditFocus::Variables {
                app.edit_component.start_editing_variable();
            }
        }
        KeyCode::Char('n') => {
            if app.edit_component.current_focus() == EditFocus::Profiles {
                open_add_dependency_popup(app);
            }
        }
        KeyCode::Char('s') | KeyCode::Char('w') => {
            // Save current profile
            let name = app.edit_component.profile_name().to_string();
            let new_profile = app.edit_component.to_profile();

            app.config_manager
                .app_config
                .profiles
                .insert(name.clone(), new_profile);

            app.list_component.mark_dirty(name.clone());

            match crate::config::graph::ProfileGraph::build(&app.config_manager.app_config.profiles)
            {
                Ok(graph) => {
                    app.config_manager.app_config.graph = graph;
                }
                Err(e) => {
                    app.status_message = Some(format!("Graph validation failed: {e}"));
                }
            }
        }
        _ => {}
    }
}

fn validate_variable_key_input(edit: &mut EditComponent) -> bool {
    if let Some(input) = edit.get_focused_variable_input_mut() {
        input.is_valid = true;
        input.error_message = None;

        if let Err(e) = validate_non_empty(&input.text) {
            input.set_error_message(&e);
            return false;
        }
        if let Err(e) = validate_no_spaces(&input.text) {
            input.set_error_message(&e);
            return false;
        }
        if let Err(e) = validate_starts_with_non_digit(&input.text) {
            input.set_error_message(&e);
            return false;
        }
        true
    } else {
        false
    }
}

fn should_delete_variable_row(edit: &EditComponent) -> bool {
    let idx = edit.selected_variable_index();
    !edit.is_variable_valid(idx)
}

fn update_profile_from_edit(app: &mut App) {
    let name = app.edit_component.profile_name().to_string();
    let new_profile = app.edit_component.to_profile();

    app.config_manager
        .app_config
        .profiles
        .insert(name.clone(), new_profile);

    app.list_component.mark_dirty(name);

    match crate::config::graph::ProfileGraph::build(&app.config_manager.app_config.profiles) {
        Ok(graph) => {
            app.config_manager.app_config.graph = graph;
        }
        Err(e) => {
            app.status_message = Some(format!("Graph validation failed: {e}"));
        }
    }
}

fn mark_if_changed(app: &mut App) {
    if app.edit_component.has_changes() {
        let name = app.edit_component.profile_name().to_string();
        app.list_component.mark_dirty(name);
    }
}

fn open_add_dependency_popup(app: &mut App) {
    let current_profile_name = app.edit_component.profile_name();
    let existing_profiles = app.edit_component.profiles();

    // Get available profiles (exclude self and already added)
    let available: Vec<String> = app
        .list_component
        .all_profiles()
        .iter()
        .filter(|p| p.as_str() != current_profile_name && !existing_profiles.contains(p))
        .cloned()
        .collect();

    app.edit_component.open_dependency_selector(available);
}
