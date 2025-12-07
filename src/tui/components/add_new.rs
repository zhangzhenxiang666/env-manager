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
    // Focus control
    pub focus: AddNewFocus,

    // State for Name input
    pub name_input: Input,

    // State for Profiles section
    pub profiles_selection_index: usize,
    pub profiles_scroll_offset: usize,
    pub added_profiles: HashSet<String>,

    // State for Variables table
    pub variables: Vec<(Input, Input)>,
    pub selected_variable_index: usize,
    pub variables_scroll_offset: usize,
    pub focused_column: AddNewVariableFocus,
    pub is_editing_variable: bool,
    pub pre_edit_buffer: Option<String>,
}

impl AddNewComponent {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn reset(&mut self) {
        self.focus = AddNewFocus::default();
        self.name_input.reset();
        self.profiles_selection_index = 0;
        self.profiles_scroll_offset = 0;
        self.added_profiles.clear();
        self.variables.clear();
        self.selected_variable_index = 0;
        self.variables_scroll_offset = 0;
        self.focused_column = AddNewVariableFocus::default();
        self.is_editing_variable = false;
        self.pre_edit_buffer = None;
    }

    // Focus switching logic
    pub fn switch_focus(&mut self, forward: bool) {
        if self.is_editing_variable {
            return;
        }
        let focuses = [
            AddNewFocus::Name,
            AddNewFocus::Profiles,
            AddNewFocus::Variables,
        ];
        let current_index = focuses.iter().position(|&f| f == self.focus).unwrap_or(0);
        let new_index = if forward {
            (current_index + 1) % focuses.len()
        } else {
            (current_index + focuses.len() - 1) % focuses.len()
        };
        self.focus = focuses[new_index];
    }

    // Profile selection logic
    pub fn select_next_profile(&mut self, profiles_count: usize) {
        if profiles_count == 0 {
            self.profiles_selection_index = 0;
            return;
        }
        let new_selection = (self.profiles_selection_index + 1) % profiles_count;
        self.profiles_selection_index = new_selection;
        if new_selection == 0 {
            self.profiles_scroll_offset = 0;
        } else if new_selection >= self.profiles_scroll_offset + MAX_HEIGHT {
            self.profiles_scroll_offset += 1;
        }
    }

    pub fn select_previous_profile(&mut self, profiles_count: usize) {
        if profiles_count == 0 {
            self.profiles_selection_index = 0;
            return;
        }
        let new_selection = (self.profiles_selection_index + profiles_count - 1) % profiles_count;
        self.profiles_selection_index = new_selection;
        if new_selection < self.profiles_scroll_offset {
            self.profiles_scroll_offset = new_selection;
        } else if new_selection == profiles_count - 1 {
            self.profiles_scroll_offset = profiles_count.saturating_sub(MAX_HEIGHT);
        }
    }

    pub fn toggle_current_profile(&mut self, profile_name: String) {
        if !self.added_profiles.remove(&profile_name) {
            self.added_profiles.insert(profile_name);
        }
    }

    // Variable table logic
    pub fn add_new_variable(&mut self) {
        self.variables.push((Input::default(), Input::default()));
        self.selected_variable_index = self.variables.len() - 1;

        // Ensure the new variable is visible
        if self.selected_variable_index >= self.variables_scroll_offset + MAX_VARIABLES_HEIGHT {
            self.variables_scroll_offset = self.selected_variable_index + 1 - MAX_VARIABLES_HEIGHT;
        }

        self.focused_column = AddNewVariableFocus::Key;
        self.start_editing_variable();
    }

    pub fn delete_selected_variable(&mut self) {
        if !self.variables.is_empty() && self.selected_variable_index < self.variables.len() {
            self.variables.remove(self.selected_variable_index);

            // Adjust selection if it went out of bounds
            if self.selected_variable_index >= self.variables.len() && !self.variables.is_empty() {
                self.selected_variable_index = self.variables.len() - 1;
            } else if self.variables.is_empty() {
                self.selected_variable_index = 0;
            }

            // Adjust scroll offset
            // If the list is now smaller than the current offset, we must scroll up
            // Example: offset 5, count 4 -> offset must be at most 0 or whatever fits.
            // Actually simpler: make sure the selected index is visible.
            if self.selected_variable_index < self.variables_scroll_offset {
                self.variables_scroll_offset = self.selected_variable_index;
            }
            // Also if we deleted items and the list fits within the view but offset is high (gap at top?)
            // Usually standard List behavior handles this by just keeping selection visible.
            // Let's ensure selection is visible:
            // Top check:
            if self.selected_variable_index < self.variables_scroll_offset {
                self.variables_scroll_offset = self.selected_variable_index;
            }
            // Bottom check is less likely on delete unless we were at bottom,
            // but let's re-verify:
            // If the *current* selection is too far down (impossible if we just deleted?)
            // More likely: we deleted the last item, so selection moved up.
            // If selection is now 4, offset 0, max 5, we are good.
        }
    }

    pub fn select_next_variable(&mut self) {
        if self.variables.is_empty() {
            self.selected_variable_index = 0;
            return;
        }
        let max_index = self.variables.len() - 1;
        if self.selected_variable_index < max_index {
            self.selected_variable_index += 1;
            if self.selected_variable_index >= self.variables_scroll_offset + MAX_VARIABLES_HEIGHT {
                self.variables_scroll_offset += 1;
            }
        } else {
            self.selected_variable_index = 0; // Wrap around
            self.variables_scroll_offset = 0;
        }
    }

    pub fn select_previous_variable(&mut self) {
        if self.variables.is_empty() {
            self.selected_variable_index = 0;
            return;
        }
        if self.selected_variable_index > 0 {
            self.selected_variable_index -= 1;
            if self.selected_variable_index < self.variables_scroll_offset {
                self.variables_scroll_offset = self.selected_variable_index;
            }
        } else {
            self.selected_variable_index = self.variables.len() - 1; // Wrap around
            self.variables_scroll_offset =
                self.variables.len().saturating_sub(MAX_VARIABLES_HEIGHT);
        }
    }

    pub fn switch_variable_column(&mut self) {
        self.focused_column = match self.focused_column {
            AddNewVariableFocus::Key => AddNewVariableFocus::Value,
            AddNewVariableFocus::Value => AddNewVariableFocus::Key,
        };
    }

    pub fn start_editing_variable(&mut self) {
        if !self.variables.is_empty() && !self.is_editing_variable {
            let (key, val) = &self.variables[self.selected_variable_index];
            let buffer = match self.focused_column {
                AddNewVariableFocus::Key => key.text.clone(),
                AddNewVariableFocus::Value => val.text.clone(),
            };
            self.pre_edit_buffer = Some(buffer);
            self.is_editing_variable = true;
        }
    }

    pub fn confirm_editing_variable(&mut self) {
        if self.is_editing_variable {
            self.is_editing_variable = false;
            self.pre_edit_buffer = None;
        }
    }

    pub fn cancel_editing_variable(&mut self) {
        if self.is_editing_variable {
            if let Some(buffer) = self.pre_edit_buffer.take() {
                if let Some(input) = self.get_focused_variable_input_mut() {
                    input.text = buffer;
                    input.cursor_position = input.text.chars().count();
                    input.is_valid = true;
                    input.error_message = None;
                }
            }
            self.is_editing_variable = false;
        }
    }

    pub fn get_focused_variable_input_mut(&mut self) -> Option<&mut Input> {
        if self.is_editing_variable && self.selected_variable_index < self.variables.len() {
            let (key_input, value_input) = &mut self.variables[self.selected_variable_index];
            match self.focused_column {
                AddNewVariableFocus::Key => Some(key_input),
                AddNewVariableFocus::Value => Some(value_input),
            }
        } else {
            None
        }
    }

    pub fn get_focused_variable_input(&self) -> Option<&Input> {
        if self.is_editing_variable && self.selected_variable_index < self.variables.len() {
            let (key_input, value_input) = &self.variables[self.selected_variable_index];
            match self.focused_column {
                AddNewVariableFocus::Key => Some(key_input),
                AddNewVariableFocus::Value => Some(value_input),
            }
        } else {
            None
        }
    }
}
