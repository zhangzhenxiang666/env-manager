use std::collections::HashSet;

use crate::tui::utils::Input;

pub const MAX_HEIGHT: usize = 4;
pub const MAX_VARIABLES_HEIGHT: usize = 8;

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum AddNewFocus {
    #[default]
    Name,
    Profiles,
    Variables,
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum AddNewVariableFocus {
    #[default]
    Key,
    Value,
}

// A form to store state for the 'AddNew' popup
#[derive(Default)]
pub struct AddNewComponent {
    // All fields are private - encapsulated state
    name_input: Input,

    // Profiles section
    profiles_selection_index: usize,
    added_profiles: HashSet<String>,
    profile_scroll_offset: usize,

    // Variables section
    variables: Vec<(Input, Input)>,
    selected_variable_index: usize,
    variable_scroll_offset: usize,
    variable_column_focus: AddNewVariableFocus,
    is_editing_variable: bool,
    pre_edit_buffer: Option<String>,

    // Focus management
    focus: AddNewFocus,
}

impl AddNewComponent {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn reset(&mut self) {
        self.name_input = Input::default();
        self.profiles_selection_index = 0;
        self.added_profiles.clear();
        self.profile_scroll_offset = 0;
        self.variables.clear();
        self.selected_variable_index = 0;
        self.variable_scroll_offset = 0;
        self.variable_column_focus = AddNewVariableFocus::default();
        self.is_editing_variable = false;
        self.pre_edit_buffer = None;
        self.focus = AddNewFocus::default();
    }

    // === View State Queries ===

    pub fn current_focus(&self) -> AddNewFocus {
        self.focus
    }

    pub fn is_editing(&self) -> bool {
        self.is_editing_variable
    }

    pub fn variable_column_focus(&self) -> AddNewVariableFocus {
        self.variable_column_focus
    }

    // === Name Section ===

    pub fn name_input(&self) -> &Input {
        &self.name_input
    }

    pub fn name_input_mut(&mut self) -> &mut Input {
        &mut self.name_input
    }

    // === Focus Management ===

    pub fn switch_focus(&mut self, forward: bool) {
        self.focus = if forward {
            match self.focus {
                AddNewFocus::Name => AddNewFocus::Profiles,
                AddNewFocus::Profiles => AddNewFocus::Variables,
                AddNewFocus::Variables => AddNewFocus::Name,
            }
        } else {
            match self.focus {
                AddNewFocus::Name => AddNewFocus::Variables,
                AddNewFocus::Variables => AddNewFocus::Profiles,
                AddNewFocus::Profiles => AddNewFocus::Name,
            }
        };
    }

    // === Profiles Section ===

    pub fn profiles_selection_index(&self) -> usize {
        self.profiles_selection_index
    }

    pub fn profile_scroll_offset(&self) -> usize {
        self.profile_scroll_offset
    }

    pub fn added_profiles(&self) -> &HashSet<String> {
        &self.added_profiles
    }

    pub fn is_profile_added(&self, name: &str) -> bool {
        self.added_profiles.contains(name)
    }

    pub fn select_next_profile(&mut self, profiles_count: usize) {
        if profiles_count == 0 {
            return;
        }
        if self.profiles_selection_index < profiles_count - 1 {
            self.profiles_selection_index += 1;
            self.ensure_profile_visible();
        } else {
            self.profiles_selection_index = 0;
            self.ensure_profile_visible();
        }
    }

    pub fn select_previous_profile(&mut self, profiles_count: usize) {
        if profiles_count == 0 {
            return;
        }
        if self.profiles_selection_index > 0 {
            self.profiles_selection_index -= 1;
            self.ensure_profile_visible();
        } else {
            self.profiles_selection_index = profiles_count - 1;
            self.ensure_profile_visible();
        }
    }

    pub fn toggle_current_profile(&mut self, profile_name: String) {
        if self.added_profiles.contains(&profile_name) {
            self.added_profiles.remove(&profile_name);
        } else {
            self.added_profiles.insert(profile_name);
        }
    }

    fn ensure_profile_visible(&mut self) {
        // Simple scrolling logic: ensure selected item is visible
        // If selected is before scroll offset, scroll up
        if self.profiles_selection_index < self.profile_scroll_offset {
            self.profile_scroll_offset = self.profiles_selection_index;
        }
        // Downward scrolling will be handled during rendering
    }

    /// Calculate the adjusted scroll offset for profiles given the actual viewport height
    pub fn calculate_profile_scroll_offset(&self, visible_rows: usize) -> usize {
        let visible_rows = visible_rows.max(1);
        let mut scroll_offset = self.profile_scroll_offset;

        // If selected is beyond the visible area, adjust scroll offset
        if self.profiles_selection_index >= scroll_offset + visible_rows {
            scroll_offset = self.profiles_selection_index + 1 - visible_rows;
        }
        // If selected is before scroll offset, scroll up
        if self.profiles_selection_index < scroll_offset {
            scroll_offset = self.profiles_selection_index;
        }

        scroll_offset
    }

    // === Variables Section ===

    pub fn variables_count(&self) -> usize {
        self.variables.len()
    }

    pub fn selected_variable_index(&self) -> usize {
        self.selected_variable_index
    }

    pub fn variable_scroll_offset(&self) -> usize {
        self.variable_scroll_offset
    }

    /// Get all variables as Input pairs for rendering
    pub fn variables_for_rendering(&self) -> &[(Input, Input)] {
        &self.variables
    }

    pub fn add_new_variable(&mut self) {
        self.variables.push((Input::default(), Input::default()));
        self.selected_variable_index = self.variables.len() - 1;
        self.ensure_variable_visible();

        // Auto-start editing on Key column
        self.variable_column_focus = AddNewVariableFocus::Key;
        self.start_editing_variable();

        // Switch focus to Variables if not already
        if self.focus != AddNewFocus::Variables {
            self.focus = AddNewFocus::Variables;
        }
    }

    pub fn delete_selected_variable(&mut self) {
        if self.variables.is_empty() {
            return;
        }

        if self.selected_variable_index < self.variables.len() {
            self.variables.remove(self.selected_variable_index);

            if self.variables.is_empty() {
                self.selected_variable_index = 0;
                self.variable_scroll_offset = 0;
                self.is_editing_variable = false;
                self.pre_edit_buffer = None;
            } else {
                if self.selected_variable_index >= self.variables.len() {
                    self.selected_variable_index = self.variables.len() - 1;
                }
                self.ensure_variable_visible();
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

    pub fn switch_variable_column(&mut self) {
        self.variable_column_focus = match self.variable_column_focus {
            AddNewVariableFocus::Key => AddNewVariableFocus::Value,
            AddNewVariableFocus::Value => AddNewVariableFocus::Key,
        };
    }

    pub fn start_editing_variable(&mut self) {
        if self.variables.is_empty() {
            return;
        }

        self.is_editing_variable = true;
        let (k, v) = &self.variables[self.selected_variable_index];
        self.pre_edit_buffer = Some(match self.variable_column_focus {
            AddNewVariableFocus::Key => k.text.clone(),
            AddNewVariableFocus::Value => v.text.clone(),
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
                    input.text = buf;
                    input.cursor_position = input.text.len();
                }
            }
            self.is_editing_variable = false;
        }
    }

    pub fn get_focused_variable_input_mut(&mut self) -> Option<&mut Input> {
        if self.selected_variable_index < self.variables.len() {
            let (k, v) = &mut self.variables[self.selected_variable_index];
            match self.variable_column_focus {
                AddNewVariableFocus::Key => Some(k),
                AddNewVariableFocus::Value => Some(v),
            }
        } else {
            None
        }
    }

    pub fn get_focused_variable_input(&self) -> Option<&Input> {
        if self.selected_variable_index < self.variables.len() {
            let (k, v) = &self.variables[self.selected_variable_index];
            match self.variable_column_focus {
                AddNewVariableFocus::Key => Some(k),
                AddNewVariableFocus::Value => Some(v),
            }
        } else {
            None
        }
    }

    /// Check if the variable at index is valid (for deletion logic)
    pub fn is_variable_valid(&self, index: usize) -> bool {
        if let Some((key_input, _)) = self.variables.get(index) {
            !key_input.text.is_empty()
                && !key_input.text.chars().any(char::is_whitespace)
                && !key_input
                    .text
                    .chars()
                    .next()
                    .map_or(false, |c| c.is_ascii_digit())
        } else {
            false
        }
    }

    fn ensure_variable_visible(&mut self) {
        // Simple scrolling logic: ensure selected item is visible
        // If selected is before scroll offset, scroll up
        if self.selected_variable_index < self.variable_scroll_offset {
            self.variable_scroll_offset = self.selected_variable_index;
        }
        // Downward scrolling will be handled during rendering
    }

    /// Calculate the adjusted scroll offset for variables given the actual viewport height
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
}
