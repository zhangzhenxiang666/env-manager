use std::collections::HashSet;

use crate::{config::models::Profile, tui::utils::Input};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

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

// Private internal state for dependency selector (formerly SelectPopup)
#[derive(Default)]
struct DependencySelector {
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

// Public render state for widgets
pub struct DependencySelectorState<'a> {
    pub title: &'static str,
    pub options: &'a [String],
    pub current_index: usize,
    pub selected_indices: &'a HashSet<usize>,
}

// Public state for variable input rendering
pub struct VariableInputState<'a> {
    pub text: &'a str,
    pub cursor_pos: usize,
    pub is_valid: bool,
    pub error: Option<&'a str>,
    pub is_key_focused: bool,
}

#[derive(Default)]
pub struct EditComponent {
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

    // Dependency selector (formerly SelectPopup)
    dependency_selector: DependencySelector,
    show_dependency_selector: bool,

    // Original state for change detection
    original_variables: Vec<(String, String)>,
    original_profiles: Vec<String>,
}

impl EditComponent {
    pub fn new() -> Self {
        Default::default()
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

    // === Config ===
    // Use conservative (small) values to ensure scrolling works correctly on all screen sizes
    // These are used as defaults in event handling when actual viewport size is unknown
    // The actual visible rows will be calculated dynamically during rendering
    pub const MAX_VARIABLES_HEIGHT: usize = 5;
    pub const MAX_PROFILES_HEIGHT: usize = 3;

    // === View State Queries ===

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
            if let Some((orig_k, orig_v)) = self.original_variables.get(i) {
                if k.text() != orig_k || v.text() != orig_v {
                    return true;
                }
            }
        }

        // Check if profiles changed
        self.profiles != self.original_profiles
    }

    // === Variables Section ===

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
        // Simple scrolling logic: ensure selected item is visible
        // If selected is before scroll offset, scroll up
        if self.selected_variable_index < self.variable_scroll_offset {
            self.variable_scroll_offset = self.selected_variable_index;
        }
        // If selected is after visible area, scroll down
        // We don't know the exact viewport height here, so we use a conservative approach:
        // Only scroll down if selected is significantly ahead
        // The exact adjustment will happen during rendering
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
            if let Some(buf) = self.pre_edit_buffer.take() {
                if let Some(input) = self.get_focused_variable_input_mut() {
                    input.set_text(buf);
                }
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

    // === Profiles Section ===

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
        // Simple scrolling logic: ensure selected item is visible
        // If selected is before scroll offset, scroll up
        if self.selected_profile_index < self.profile_scroll_offset {
            self.profile_scroll_offset = self.selected_profile_index;
        }
        // Downward scrolling will be handled during rendering
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

    // === Dependency Selector (formerly SelectPopup) ===

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
