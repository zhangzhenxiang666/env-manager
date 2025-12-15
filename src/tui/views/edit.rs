use crate::GLOBAL_PROFILE_MARK;
use crate::config::models::Profile;
use crate::tui::app::{App, AppState};
use crate::tui::theme::Theme;
use crate::tui::utils::{self, Input, validate_input};
use crate::tui::widgets::empty;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{
    Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Scrollbar,
    ScrollbarOrientation, ScrollbarState, Table, TableState,
};
use std::collections::HashSet;
use unicode_width::UnicodeWidthStr;

// ==================================================================================
// STATE
// ==================================================================================

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum EditFocus {
    #[default]
    Variables,
    Profiles,
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum EditVariableFocus {
    #[default]
    Key,
    Value,
}

#[derive(Default)]
pub struct DependencySelector {
    options: Vec<String>,
    current_index: usize,
    selected_indices: HashSet<usize>,
}

impl DependencySelector {
    fn new() -> Self {
        Self {
            options: Vec::new(),
            current_index: 0,
            selected_indices: HashSet::new(),
        }
    }

    fn reset(&mut self) {
        self.options.clear();
        self.current_index = 0;
        self.selected_indices.clear();
    }

    fn select_next(&mut self) {
        if self.options.is_empty() {
            return;
        }
        if self.current_index < self.options.len() - 1 {
            self.current_index += 1;
        } else {
            self.current_index = 0;
        }
    }

    fn select_previous(&mut self) {
        if self.options.is_empty() {
            return;
        }
        if self.current_index > 0 {
            self.current_index -= 1;
        } else {
            self.current_index = self.options.len() - 1;
        }
    }

    fn toggle_selection(&mut self) {
        if self.options.is_empty() {
            return;
        }

        if self.selected_indices.contains(&self.current_index) {
            self.selected_indices.remove(&self.current_index);
        } else {
            self.selected_indices.insert(self.current_index);
        }
    }

    fn get_selected_items(&self) -> Vec<String> {
        let mut indices: Vec<_> = self.selected_indices.iter().cloned().collect();
        indices.sort();
        indices.iter().map(|&i| self.options[i].clone()).collect()
    }
}

pub struct DependencySelectorState<'a> {
    pub title: &'static str,
    pub options: &'a [String],
    pub current_index: usize,
    pub selected_indices: &'a HashSet<usize>,
}

pub struct VariableInputState<'a> {
    pub text: &'a str,
    pub cursor_pos: usize,
    pub is_valid: bool,
    pub error: Option<&'a str>,
    pub is_key_focused: bool,
}

#[derive(Default)]
pub struct EditView {
    // Focus and Navigation
    focus: EditFocus,

    // Variables section
    variables: Vec<(Input, Input)>,
    selected_variable_index: usize,
    variable_scroll_offset: usize,
    variable_column_focus: EditVariableFocus,
    is_editing_variable: bool,
    pre_edit_buffer: Option<String>,

    // Profiles (dependencies) section
    profiles: Vec<String>,
    selected_profile_index: usize,
    profile_scroll_offset: usize,

    // Profile name (for display)
    profile_name: String,

    // Dependency selector
    dependency_selector: DependencySelector,
    show_dependency_selector: bool,

    // Original state for change detection
    original_variables: Vec<(String, String)>,
    original_profiles: Vec<String>,
}

impl EditView {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn reset(&mut self) {
        self.variables.clear();
        self.pre_edit_buffer.take();
        self.profiles.clear();
        self.profile_name.clear();
        self.dependency_selector.reset();
        self.original_profiles.clear();
        self.original_variables.clear();
    }

    pub fn from_profile(name: &str, profile: &Profile) -> Self {
        // Convert map to vec for editable inputs
        let mut variables: Vec<(Input, Input)> = profile
            .variables
            .iter()
            .map(|(k, v)| {
                let k_in = Input::with_text(k.clone());
                let v_in = Input::with_text(v.clone());
                (k_in, v_in)
            })
            .collect();
        variables.sort_by(|a, b| a.0.text().cmp(b.0.text()));

        let mut profiles: Vec<String> = profile.profiles.iter().cloned().collect();
        profiles.sort();

        // Create snapshots of original state for change detection
        let original_variables: Vec<(String, String)> = variables
            .iter()
            .map(|(k, v)| (k.text().to_string(), v.text().to_string()))
            .collect();
        let original_profiles = profiles.clone();

        Self {
            focus: EditFocus::Variables,
            variables,
            selected_variable_index: 0,
            variable_scroll_offset: 0,
            variable_column_focus: EditVariableFocus::Key,
            is_editing_variable: false,
            pre_edit_buffer: None,
            profiles,
            selected_profile_index: 0,
            profile_scroll_offset: 0,
            profile_name: name.to_string(),
            dependency_selector: DependencySelector::new(),
            show_dependency_selector: false,
            original_variables,
            original_profiles,
        }
    }

    pub fn to_profile(&self) -> Profile {
        let mut variables_map = std::collections::HashMap::new();
        for (k, v) in &self.variables {
            if !k.text().is_empty() {
                variables_map.insert(k.text().to_string(), v.text().to_string());
            }
        }

        Profile {
            variables: variables_map,
            profiles: self.profiles.iter().cloned().collect(),
        }
    }

    pub fn current_focus(&self) -> EditFocus {
        self.focus
    }

    pub fn is_editing(&self) -> bool {
        self.is_editing_variable
    }

    pub fn profile_name(&self) -> &str {
        &self.profile_name
    }

    /// Check if there are any unsaved changes compared to original state
    pub fn has_changes(&self) -> bool {
        // Check if variables count changed
        if self.variables.len() != self.original_variables.len() {
            return true;
        }

        // Check if any variable content changed
        for (i, (k, v)) in self.variables.iter().enumerate() {
            if let Some((orig_k, orig_v)) = self.original_variables.get(i)
                && (k.text() != orig_k || v.text() != orig_v)
            {
                return true;
            }
        }

        // Check if profiles changed
        self.profiles != self.original_profiles
    }

    /// Get iterator over variables (key, value) pairs for rendering
    pub fn variables(&self) -> impl Iterator<Item = (&str, &str)> {
        self.variables.iter().map(|(k, v)| (k.text(), v.text()))
    }

    pub fn variables_count(&self) -> usize {
        self.variables.len()
    }

    pub fn selected_variable_index(&self) -> usize {
        self.selected_variable_index
    }

    pub fn variable_scroll_offset(&self) -> usize {
        self.variable_scroll_offset
    }

    pub fn variable_column_focus(&self) -> EditVariableFocus {
        self.variable_column_focus
    }

    /// Get the current variable input state for rendering
    pub fn variable_input_state(&self) -> Option<VariableInputState<'_>> {
        if !self.is_editing_variable || self.selected_variable_index >= self.variables.len() {
            return None;
        }

        let (k, v) = &self.variables[self.selected_variable_index];
        let is_key_focused = self.variable_column_focus == EditVariableFocus::Key;
        let input = if is_key_focused { k } else { v };

        Some(VariableInputState {
            text: input.text(),
            cursor_pos: input.cursor_position(),
            is_valid: input.is_valid(),
            error: input.error_message(),
            is_key_focused,
        })
    }

    /// Get all variables as Input pairs for rendering table
    pub fn variables_for_rendering(&self) -> &[(Input, Input)] {
        &self.variables
    }

    pub fn add_variable(&mut self) {
        self.variables.push((Input::default(), Input::default()));
        self.selected_variable_index = self.variables.len() - 1;
        self.ensure_variable_visible();
        self.variable_column_focus = EditVariableFocus::Key;
        self.start_editing_variable();
    }

    pub fn delete_variable(&mut self) {
        if !self.variables.is_empty() && self.selected_variable_index < self.variables.len() {
            self.variables.remove(self.selected_variable_index);
            if self.selected_variable_index >= self.variables.len() && !self.variables.is_empty() {
                self.selected_variable_index = self.variables.len() - 1;
            } else if self.variables.is_empty() {
                self.selected_variable_index = 0;
            }
        }
    }

    pub fn select_next_variable(&mut self) {
        if self.variables.is_empty() {
            return;
        }
        if self.selected_variable_index < self.variables.len() - 1 {
            self.selected_variable_index += 1;
            self.ensure_variable_visible();
        } else {
            self.selected_variable_index = 0;
            self.ensure_variable_visible();
        }
    }

    pub fn select_previous_variable(&mut self) {
        if self.variables.is_empty() {
            return;
        }
        if self.selected_variable_index > 0 {
            self.selected_variable_index -= 1;
            self.ensure_variable_visible();
        } else {
            self.selected_variable_index = self.variables.len() - 1;
            self.ensure_variable_visible();
        }
    }

    fn ensure_variable_visible(&mut self) {
        if self.selected_variable_index < self.variable_scroll_offset {
            self.variable_scroll_offset = self.selected_variable_index;
        }
    }

    /// Calculate the adjusted scroll offset to ensure selected item is visible
    /// given the actual viewport height. Returns the scroll offset to use for rendering.
    pub fn calculate_variable_scroll_offset(&self, visible_rows: usize) -> usize {
        let visible_rows = visible_rows.max(1);
        let mut scroll_offset = self.variable_scroll_offset;

        // If selected is beyond the visible area, adjust scroll offset
        if self.selected_variable_index >= scroll_offset + visible_rows {
            scroll_offset = self.selected_variable_index + 1 - visible_rows;
        }
        // If selected is before scroll offset, scroll up
        if self.selected_variable_index < scroll_offset {
            scroll_offset = self.selected_variable_index;
        }

        scroll_offset
    }

    pub fn switch_variable_column(&mut self) {
        self.variable_column_focus = match self.variable_column_focus {
            EditVariableFocus::Key => EditVariableFocus::Value,
            EditVariableFocus::Value => EditVariableFocus::Key,
        };
    }

    pub fn start_editing_variable(&mut self) {
        if self.variables.is_empty() {
            return;
        }

        self.is_editing_variable = true;
        let (k, v) = &self.variables[self.selected_variable_index];
        self.pre_edit_buffer = Some(match self.variable_column_focus {
            EditVariableFocus::Key => k.text().to_string(),
            EditVariableFocus::Value => v.text().to_string(),
        });
    }

    pub fn confirm_editing_variable(&mut self) {
        self.is_editing_variable = false;
        self.pre_edit_buffer = None;
    }

    pub fn cancel_editing_variable(&mut self) {
        if self.is_editing_variable {
            if let Some(buf) = self.pre_edit_buffer.take()
                && let Some(input) = self.get_focused_variable_input_mut()
            {
                input.set_text(buf);
            }
            self.is_editing_variable = false;
        }
    }

    pub fn get_focused_variable_input_mut(&mut self) -> Option<&mut Input> {
        if self.selected_variable_index < self.variables.len() {
            let (k, v) = &mut self.variables[self.selected_variable_index];
            match self.variable_column_focus {
                EditVariableFocus::Key => Some(k),
                EditVariableFocus::Value => Some(v),
            }
        } else {
            None
        }
    }

    /// Check if the variable at index is valid (for deletion logic)
    pub fn is_variable_valid(&self, index: usize) -> bool {
        if let Some((key_input, _)) = self.variables.get(index) {
            !key_input.text().is_empty()
                && !key_input.text().chars().any(char::is_whitespace)
                && !key_input
                    .text()
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_digit())
        } else {
            false
        }
    }

    pub fn profiles(&self) -> &[String] {
        &self.profiles
    }

    pub fn profiles_count(&self) -> usize {
        self.profiles.len()
    }

    pub fn selected_profile_index(&self) -> usize {
        self.selected_profile_index
    }

    pub fn profile_scroll_offset(&self) -> usize {
        self.profile_scroll_offset
    }

    pub fn add_profile_dependency(&mut self, name: String) {
        if !self.profiles.contains(&name) {
            self.profiles.push(name);
            self.profiles.sort();
        }
    }

    pub fn remove_profile_dependency(&mut self) {
        if !self.profiles.is_empty() && self.selected_profile_index < self.profiles.len() {
            self.profiles.remove(self.selected_profile_index);
            if self.selected_profile_index >= self.profiles.len() && !self.profiles.is_empty() {
                self.selected_profile_index = self.profiles.len() - 1;
            } else if self.profiles.is_empty() {
                self.selected_profile_index = 0;
            }
        }
    }

    pub fn select_next_profile(&mut self) {
        if self.profiles.is_empty() {
            return;
        }
        if self.selected_profile_index < self.profiles.len() - 1 {
            self.selected_profile_index += 1;
            self.ensure_profile_visible();
        } else {
            self.selected_profile_index = 0;
            self.ensure_profile_visible();
        }
    }

    pub fn select_previous_profile(&mut self) {
        if self.profiles.is_empty() {
            return;
        }
        if self.selected_profile_index > 0 {
            self.selected_profile_index -= 1;
            self.ensure_profile_visible();
        } else {
            self.selected_profile_index = self.profiles.len() - 1;
            self.ensure_profile_visible();
        }
    }

    fn ensure_profile_visible(&mut self) {
        if self.selected_profile_index < self.profile_scroll_offset {
            self.profile_scroll_offset = self.selected_profile_index;
        }
    }

    /// Calculate the adjusted scroll offset for profiles given the actual viewport height
    pub fn calculate_profile_scroll_offset(&self, visible_rows: usize) -> usize {
        let visible_rows = visible_rows.max(1);
        let mut scroll_offset = self.profile_scroll_offset;

        // If selected is beyond the visible area, adjust scroll offset
        if self.selected_profile_index >= scroll_offset + visible_rows {
            scroll_offset = self.selected_profile_index + 1 - visible_rows;
        }
        // If selected is before scroll offset, scroll up
        if self.selected_profile_index < scroll_offset {
            scroll_offset = self.selected_profile_index;
        }

        scroll_offset
    }

    pub fn switch_focus(&mut self) {
        self.focus = match self.focus {
            EditFocus::Variables => EditFocus::Profiles,
            EditFocus::Profiles => EditFocus::Variables,
        };
    }

    pub fn is_dependency_selector_open(&self) -> bool {
        self.show_dependency_selector
    }

    pub fn dependency_selector_state(&self) -> Option<DependencySelectorState<'_>> {
        if !self.show_dependency_selector {
            return None;
        }

        Some(DependencySelectorState {
            title: "Add Dependency",
            options: &self.dependency_selector.options,
            current_index: self.dependency_selector.current_index,
            selected_indices: &self.dependency_selector.selected_indices,
        })
    }

    pub fn open_dependency_selector(&mut self, available: Vec<String>) {
        if self.focus != EditFocus::Profiles {
            return;
        }

        self.dependency_selector.reset();
        self.dependency_selector.options = available;
        self.show_dependency_selector = true;
    }

    pub fn close_dependency_selector(&mut self) {
        self.show_dependency_selector = false;
        self.dependency_selector.reset();
    }

    /// Handle input for dependency selector, returns selected items if Esc pressed to confirm
    pub fn handle_selector_input(&mut self, key: KeyEvent) -> Option<Vec<String>> {
        if !self.show_dependency_selector {
            return None;
        }

        match key.code {
            KeyCode::Esc => {
                let selected = self.dependency_selector.get_selected_items();
                self.close_dependency_selector();
                Some(selected)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.dependency_selector.select_previous();
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.dependency_selector.select_next();
                None
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.dependency_selector.toggle_selection();
                None
            }
            _ => None,
        }
    }
}

// ==================================================================================
// EVENT HANDLING
// ==================================================================================

pub fn handle_event(app: &mut App, key: KeyEvent) {
    if app.edit_view.is_dependency_selector_open() {
        handle_dependency_selector(app, key);
        return;
    }

    if app.edit_view.is_editing() {
        handle_variable_editing_mode(app, key);
    } else {
        handle_navigation_mode(app, key);
    }
}

fn handle_dependency_selector(app: &mut App, key: KeyEvent) {
    if let Some(selected_deps) = app.edit_view.handle_selector_input(key) {
        add_dependencies_to_profile(app, selected_deps);
    }
}

fn add_dependencies_to_profile(app: &mut App, dep_names: Vec<String>) {
    let profile_name = app.edit_view.profile_name().to_string();
    if profile_name == GLOBAL_PROFILE_MARK {
        dep_names
            .into_iter()
            .for_each(|name| app.edit_view.add_profile_dependency(name));
    } else {
        for dep_name in dep_names {
            // Try to add to graph first (validation)
            match app
                .config_manager
                .add_dependency_edge(&profile_name, &dep_name)
            {
                Ok(_) => {
                    // Success: update UI component
                    app.edit_view.add_profile_dependency(dep_name);
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
    let profile_name = app.edit_view.profile_name().to_string();
    let selected_idx = app.edit_view.selected_profile_index();
    if profile_name == GLOBAL_PROFILE_MARK {
        app.edit_view.remove_profile_dependency();
    } else if let Some(removed_dep) = app.edit_view.profiles().get(selected_idx) {
        let removed_dep = removed_dep.clone();

        // Update UI component
        app.edit_view.remove_profile_dependency();

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

fn open_dependency_selector_handler(app: &mut App) {
    let current_profile = app.edit_view.profile_name();
    let existing_deps = app.edit_view.profiles();

    // Get profiles that depend on current (would create cycle)
    let ancestors: std::collections::HashSet<String> = app
        .config_manager
        .get_parents(current_profile)
        .unwrap_or_default()
        .into_iter()
        .collect();

    // Filter available profiles
    let available: Vec<String> = app
        .list_view
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

    app.edit_view.open_dependency_selector(available);
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
        let edit = &mut app.edit_view;

        // Validate Key before confirming
        if edit.variable_column_focus() == EditVariableFocus::Key && !validate_variable_key(edit) {
            return;
        }

        edit.confirm_editing_variable();
        edit.variable_column_focus() == EditVariableFocus::Key
    };

    mark_profile_as_dirty_if_changed(app);

    if should_switch {
        app.edit_view.switch_variable_column();
        app.edit_view.start_editing_variable();
    }
}

fn confirm_and_switch_column(app: &mut App) {
    {
        let edit = &mut app.edit_view;

        // Validate Key before switching
        if edit.variable_column_focus() == EditVariableFocus::Key && !validate_variable_key(edit) {
            return;
        }

        edit.confirm_editing_variable();
    }

    mark_profile_as_dirty_if_changed(app);

    app.edit_view.switch_variable_column();
    app.edit_view.start_editing_variable();
}

fn cancel_variable_editing(app: &mut App) {
    let edit = &mut app.edit_view;
    edit.cancel_editing_variable();

    // Delete row if invalid (empty key, etc.)
    if should_delete_invalid_variable(edit) {
        edit.delete_variable();
    }
}

fn handle_text_input(app: &mut App, key_code: KeyCode) {
    let edit = &mut app.edit_view;

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
        KeyCode::Tab => app.edit_view.switch_focus(),

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
    if app.list_view.is_dirty(app.edit_view.profile_name()) {
        save_profile_to_memory(app);
    }
    app.state = AppState::List;
    app.edit_view.reset();
}

fn navigate_down(app: &mut App) {
    match app.edit_view.current_focus() {
        EditFocus::Variables => app.edit_view.select_next_variable(),
        EditFocus::Profiles => app.edit_view.select_next_profile(),
    }
}

fn navigate_up(app: &mut App) {
    match app.edit_view.current_focus() {
        EditFocus::Variables => app.edit_view.select_previous_variable(),
        EditFocus::Profiles => app.edit_view.select_previous_profile(),
    }
}

fn switch_column_if_in_variables(app: &mut App) {
    if app.edit_view.current_focus() == EditFocus::Variables {
        app.edit_view.switch_variable_column();
    }
}

fn add_variable_if_in_variables(app: &mut App) {
    if app.edit_view.current_focus() == EditFocus::Variables {
        app.edit_view.add_variable();
        mark_profile_as_dirty_if_changed(app);
    }
}

fn delete_current_item(app: &mut App) {
    match app.edit_view.current_focus() {
        EditFocus::Variables => {
            app.edit_view.delete_variable();
            mark_profile_as_dirty_if_changed(app);
        }
        EditFocus::Profiles => {
            remove_dependency_from_profile(app);
        }
    }
}

fn start_editing_variable_if_in_variables(app: &mut App) {
    if app.edit_view.current_focus() == EditFocus::Variables {
        app.edit_view.start_editing_variable();
    }
}

fn open_dependency_selector_if_in_profiles(app: &mut App) {
    if app.edit_view.current_focus() == EditFocus::Profiles {
        open_dependency_selector_handler(app);
    }
}

/// Validate variable key (non-empty, no spaces, not start with digit)
fn validate_variable_key(edit: &mut EditView) -> bool {
    if let Some(input) = edit.get_focused_variable_input_mut() {
        input.clear_error();
        validate_input(input)
    } else {
        true
    }
}

/// Check if current variable row is invalid and should be deleted
fn should_delete_invalid_variable(edit: &EditView) -> bool {
    let idx = edit.selected_variable_index();
    !edit.is_variable_valid(idx)
}

/// Save edited profile to memory (called on Esc)
fn save_profile_to_memory(app: &mut App) {
    let name = app.edit_view.profile_name().to_string();
    let new_profile = app.edit_view.to_profile();

    // Update profile in memory
    app.config_manager
        .add_profile(name.clone(), new_profile.clone());

    if name == GLOBAL_PROFILE_MARK {
        if let Err(e) = app.config_manager.write_global(&new_profile) {
            app.status_message = Some(format!("Error saving GLOBAL: {}", e));
        } else {
            app.list_view.clear_dirty(&name);
        }
    } else {
        app.list_view.mark_dirty(name);
    }
}

/// Mark profile as dirty if there are any changes
fn mark_profile_as_dirty_if_changed(app: &mut App) {
    if app.edit_view.has_changes() {
        let name = app.edit_view.profile_name().to_string();
        app.list_view.mark_dirty(name);
    }
}

// ==================================================================================
// RENDERING
// ==================================================================================

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = Theme::new();
    let edit = &app.edit_view;
    let profile_name = edit.profile_name();
    let title = format!("Editing '{profile_name}'");

    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.block_active())
        .border_type(ratatui::widgets::BorderType::Thick)
        .title_top(
            Line::from(title)
                .left_aligned()
                .style(theme.block_title_active()),
        );

    let inner_area = main_block.inner(area);
    frame.render_widget(main_block, area);

    // Vertical Layout: Profiles Top (30%), Variables Bottom (70%)
    let chunks = Layout::vertical([
        Constraint::Percentage(30), // Inherited Profiles
        Constraint::Percentage(70), // Variables
    ])
    .split(inner_area);

    let profiles_area = chunks[0];
    let variables_area = chunks[1];

    // Calculate actual visible rows for variables area
    let variables_inner_height = variables_area.height.saturating_sub(2) as usize;
    let actual_visible_rows = variables_inner_height.saturating_sub(2).max(1);

    let vars_focus = edit.current_focus() == EditFocus::Variables;
    let profiles_focus = edit.current_focus() == EditFocus::Profiles;

    // --- PROFILES SECTION ---
    let current_prof_idx = if edit.profiles_count() == 0 {
        0
    } else {
        edit.selected_profile_index() + 1
    };
    let profiles_title = format!(
        "Inherited Profiles ({}/{})",
        current_prof_idx,
        edit.profiles_count()
    );

    let prof_border_style = if profiles_focus {
        theme.block_active()
    } else {
        theme.block_inactive()
    };

    let actual_visible_profiles = profiles_area.height.saturating_sub(2) as usize; // Remove borders
    let render_profile_scroll = edit.calculate_profile_scroll_offset(actual_visible_profiles);

    let profile_items: Vec<ListItem> = edit
        .profiles()
        .iter()
        .skip(render_profile_scroll)
        .map(|p| ListItem::new(p.as_str()))
        .collect();

    let is_empty = profile_items.is_empty();

    let profiles_list = List::new(profile_items).block(
        Block::new()
            .title(profiles_title)
            .borders(Borders::ALL)
            .border_style(prof_border_style),
    );

    let profiles_list = if profiles_focus {
        profiles_list.highlight_style(theme.row_selected())
    } else {
        profiles_list
    };

    if is_empty {
        empty::profile_not_inherited(frame, profiles_area);
    }

    let mut list_state = ListState::default();
    list_state.select(Some(edit.selected_profile_index()));

    frame.render_stateful_widget(profiles_list, profiles_area, &mut list_state);

    // Scrollbar for profiles
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .symbols(ratatui::symbols::scrollbar::VERTICAL)
        .begin_symbol(None)
        .end_symbol(None);

    let max_scroll = edit
        .profiles_count()
        .saturating_sub(actual_visible_profiles)
        + 1;
    let mut scrollbar_state = ScrollbarState::new(max_scroll).position(render_profile_scroll);

    frame.render_stateful_widget(
        scrollbar,
        profiles_area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );

    // --- VARIABLES SECTION ---
    let current_var_idx = if edit.variables_count() == 0 {
        0
    } else {
        edit.selected_variable_index() + 1
    };
    let vars_title = format!("Variables ({}/{})", current_var_idx, edit.variables_count());

    let vars_border_style = if vars_focus && !edit.is_editing() {
        theme.block_active()
    } else {
        theme.block_inactive()
    };

    let variables_block = Block::default()
        .title_top(Line::from(vars_title).left_aligned())
        .borders(Borders::ALL)
        .border_style(vars_border_style);

    let header = Row::new(vec!["Key", "Value"])
        .style(Style::new().add_modifier(Modifier::BOLD))
        .style(theme.text_highlight())
        .bottom_margin(1);

    let variable_rows: Vec<Row> = edit
        .variables_for_rendering()
        .iter()
        .enumerate()
        .map(|(idx, (k, v))| {
            let key_text = k.text();
            let value_text = v.text();
            let selected = idx == edit.selected_variable_index();

            let (key_style, value_style) = if selected && vars_focus {
                match edit.variable_column_focus() {
                    EditVariableFocus::Key => (theme.cell_focus(), theme.selection_active()),
                    EditVariableFocus::Value => (theme.selection_active(), theme.cell_focus()),
                }
            } else {
                (theme.text_normal(), theme.text_normal())
            };

            Row::new(vec![
                Cell::from(key_text).style(key_style),
                Cell::from(value_text).style(value_style),
            ])
        })
        .collect();

    let is_empty = variable_rows.is_empty();
    let render_scroll_offset = edit.calculate_variable_scroll_offset(actual_visible_rows);

    let mut table_state = TableState::default().with_offset(render_scroll_offset);
    if vars_focus && !edit.variables_for_rendering().is_empty() {
        table_state.select(Some(edit.selected_variable_index()));
    }

    let col_widths = [Constraint::Percentage(30), Constraint::Percentage(70)];
    let table = Table::new(variable_rows, col_widths)
        .header(header)
        .block(variables_block.clone());

    if is_empty {
        empty::variable_not_defined(frame, variables_area);
    }

    frame.render_stateful_widget(table, variables_area, &mut table_state);

    // Scrollbar for variables
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .symbols(ratatui::symbols::scrollbar::VERTICAL)
        .begin_symbol(None)
        .end_symbol(None);

    let max_scroll = edit.variables_count().saturating_sub(actual_visible_rows) + 1;
    let mut scrollbar_state = ScrollbarState::new(max_scroll).position(render_scroll_offset);

    frame.render_stateful_widget(
        scrollbar,
        variables_area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );

    // Render variable input popup if editing
    if edit.is_editing()
        && let Some(input_state) = edit.variable_input_state()
    {
        let table_inner_area = variables_block.inner(variables_area);

        let vis_idx = edit
            .selected_variable_index()
            .saturating_sub(render_scroll_offset);

        let row_y = table_inner_area.y + 2 + vis_idx as u16;
        let is_key_focused = input_state.is_key_focused;
        let col_index = if is_key_focused { 0 } else { 1 };

        let layout = Layout::horizontal(col_widths).spacing(1);
        let column_chunks = layout.split(table_inner_area);
        let cell_area = column_chunks[col_index];

        let popup_area = Rect {
            x: cell_area.x.saturating_sub(1),
            y: row_y.saturating_sub(1),
            width: cell_area.width + 2,
            height: 3,
        };

        let title = if is_key_focused {
            "Edit Variable"
        } else {
            "Edit Value"
        };

        let temp_input = Input::from_parts(
            input_state.text.to_string(),
            input_state.cursor_pos,
            input_state.error.map(|s| s.to_string()),
        );

        render_variable_input_popup(frame, popup_area, &temp_input, title, &theme);
    }

    // Render dependency selector if open
    if edit.is_dependency_selector_open()
        && let Some(selector_state) = edit.dependency_selector_state()
    {
        render_dependency_selector(frame, selector_state, &theme);
    }
}

fn render_variable_input_popup(
    frame: &mut Frame,
    area: Rect,
    input: &Input,
    title: &str,
    theme: &Theme,
) {
    frame.render_widget(Clear, area);

    let border_style = if input.is_valid() {
        theme.block_active()
    } else {
        theme.text_error()
    };

    let mut block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    if !input.is_valid()
        && let Some(err) = input.error_message()
    {
        block = block.title_bottom(Line::from(err).style(theme.text_error()).right_aligned());
    }

    let inner_area = block.inner(area);

    let text = input.text();
    let cursor_pos = input.cursor_position();

    let prefix_width = text
        .chars()
        .take(cursor_pos)
        .map(|c| UnicodeWidthStr::width(c.to_string().as_str()))
        .sum::<usize>();

    let cursor_display_pos = prefix_width as u16;
    let scroll_offset = if cursor_display_pos >= inner_area.width {
        cursor_display_pos - inner_area.width + 1
    } else {
        0
    };

    let paragraph = Paragraph::new(text).scroll((0, scroll_offset));

    frame.render_widget(block, area);
    frame.render_widget(paragraph, inner_area);
    frame.set_cursor_position((
        inner_area.x + cursor_display_pos - scroll_offset,
        inner_area.y,
    ));
}

fn render_dependency_selector(
    frame: &mut Frame,
    selector_state: DependencySelectorState,
    theme: &Theme,
) {
    let area = utils::centered_rect(60, 60, frame.area());
    frame.render_widget(Clear, area);

    let outer_block = Block::default()
        .title(selector_state.title)
        .borders(Borders::ALL)
        .border_style(theme.block_active())
        .border_type(ratatui::widgets::BorderType::Thick);

    let inner_area = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let chunks = Layout::vertical([
        Constraint::Min(0),    // List area
        Constraint::Length(2), // Help section
    ])
    .split(inner_area);

    let list_area = chunks[0];
    let help_area = chunks[1];

    let items: Vec<ListItem> = selector_state
        .options
        .iter()
        .enumerate()
        .map(|(idx, name)| {
            let selected = selector_state.selected_indices.contains(&idx);
            let marker = if selected { "[✓] " } else { "[ ] " };
            ListItem::new(format!("{marker}{name}"))
        })
        .collect();

    let is_empty = items.is_empty();

    let current_pos = if selector_state.options.is_empty() {
        0
    } else {
        selector_state.current_index + 1
    };
    let total_count = selector_state.options.len();
    let selected_count = selector_state.selected_indices.len();

    let left_title = Line::from(format!("{current_pos}/{total_count}")).left_aligned();
    let right_title = Line::from(format!("Selected: {selected_count}")).right_aligned();

    let list = List::new(items)
        .block(
            Block::default()
                .title_top(left_title)
                .title_top(right_title)
                .borders(Borders::ALL)
                .border_style(theme.block_inactive()),
        )
        .highlight_style(theme.row_selected());

    let mut list_state = ListState::default();
    list_state.select(Some(selector_state.current_index));

    if is_empty {
        empty::profile_not_selectable(frame, list_area);
    }

    frame.render_stateful_widget(list, list_area, &mut list_state);

    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .symbols(ratatui::symbols::scrollbar::VERTICAL)
        .begin_symbol(None)
        .end_symbol(None);

    let inner_height = list_area.height.saturating_sub(2) as usize;
    let actual_visible = inner_height.max(1);
    let max_scroll = selector_state.options.len().saturating_sub(actual_visible) + 1;

    let mut scrollbar_state = ScrollbarState::new(max_scroll).position(
        selector_state
            .current_index
            .saturating_sub(actual_visible / 2)
            .min(max_scroll.saturating_sub(1)),
    );

    frame.render_stateful_widget(
        scrollbar,
        list_area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );

    let help_info = [
        vec![
            Span::styled("Esc", Style::default().fg(Color::Rgb(255, 107, 107))),
            Span::raw(": Confirm"),
        ],
        vec![
            Span::styled("↑↓", Style::default().fg(Color::Rgb(255, 138, 199))),
            Span::raw(": Navigate"),
        ],
        vec![
            Span::styled("Enter", Style::default().fg(Color::LightBlue)),
            Span::raw("/"),
            Span::styled("Space", Style::default().fg(Color::LightBlue)),
            Span::raw(": Toggle"),
        ],
    ];

    let help_spans = create_selector_help_spans(&help_info, help_area);
    let help_paragraph = Paragraph::new(help_spans).style(Style::default());
    frame.render_widget(help_paragraph, help_area);
}

fn create_selector_help_spans<'a>(help_info: &'a [Vec<Span<'a>>], area: Rect) -> Vec<Line<'a>> {
    let total_width = area.width as usize;
    let mut lines: Vec<Line> = vec![];
    let mut current_line_spans: Vec<Span> = vec![];
    let mut current_line_width = 0;
    let max_help_lines = 2;

    for info in help_info {
        if lines.len() >= max_help_lines {
            break;
        }
        let item_width: usize = info.iter().map(|span| span.width()).sum();
        let separator_width = if !current_line_spans.is_empty() { 2 } else { 0 };

        if current_line_width + separator_width + item_width > total_width
            && !current_line_spans.is_empty()
        {
            if lines.len() < max_help_lines {
                lines.push(Line::from(std::mem::take(&mut current_line_spans)));
                current_line_width = 0;
            } else {
                break;
            }
        }
        if !current_line_spans.is_empty() {
            current_line_spans.push(Span::raw("  "));
            current_line_width += 2;
        }
        current_line_spans.extend_from_slice(info);
        current_line_width += item_width;
    }
    if !current_line_spans.is_empty() && lines.len() < max_help_lines {
        lines.push(Line::from(current_line_spans));
    }
    lines
}
