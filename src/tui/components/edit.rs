use std::collections::HashMap;

use crate::{config::models::Profile, tui::utils::Input};

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
pub struct EditComponent {
    // Current focus section
    pub focus: EditFocus,

    // Variables Section
    pub variables: Vec<(Input, Input)>,
    pub selected_variable_index: usize,
    pub variable_scroll_offset: usize,
    pub variable_column_focus: EditVariableFocus,
    pub is_editing_variable: bool,
    pub pre_edit_buffer: Option<String>,

    // Profiles (Dependencies) Section
    pub profiles: Vec<String>,
    pub selected_profile_index: usize,
    pub profile_scroll_offset: usize,

    // Original profile name (to know what we are editing)
    // Original profile name (to know what we are editing)
    pub profile_name: String,

    // Popup State
    pub select_popup: crate::tui::components::select_popup::SelectPopupComponent,
    pub show_select_popup: bool,
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
                let mut k_in = Input::default();
                k_in.text = k.clone();
                k_in.cursor_position = k.len(); // Optional: set cursor at end

                let mut v_in = Input::default();
                v_in.text = v.clone();
                v_in.cursor_position = v.len();

                (k_in, v_in)
            })
            .collect();
        // Sort for consistent display
        variables.sort_by(|a, b| a.0.text.cmp(&b.0.text));

        let mut profiles: Vec<String> = profile.profiles.iter().cloned().collect();
        profiles.sort();

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
            select_popup: Default::default(),
            show_select_popup: false,
        }
    }

    pub fn to_profile(&self) -> Profile {
        let mut variables_map = HashMap::new();
        for (k, v) in &self.variables {
            if !k.text.is_empty() {
                variables_map.insert(k.text.clone(), v.text.clone());
            }
        }

        Profile {
            variables: variables_map,
            profiles: self.profiles.iter().cloned().collect(),
        }
    }

    // --- Config ---

    pub const MAX_VARIABLES_HEIGHT: usize = 8; // Adjust based on layout
    pub const MAX_PROFILES_HEIGHT: usize = 4;

    // --- Navigation & Focus ---

    pub fn switch_focus(&mut self) {
        self.focus = match self.focus {
            EditFocus::Variables => EditFocus::Profiles,
            EditFocus::Profiles => EditFocus::Variables,
        };
    }

    // --- Variables Logic ---

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
        if self.selected_variable_index >= self.variable_scroll_offset + Self::MAX_VARIABLES_HEIGHT
        {
            self.variable_scroll_offset =
                self.selected_variable_index + 1 - Self::MAX_VARIABLES_HEIGHT;
        } else if self.selected_variable_index < self.variable_scroll_offset {
            self.variable_scroll_offset = self.selected_variable_index;
        }
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
            EditVariableFocus::Key => k.text.clone(),
            EditVariableFocus::Value => v.text.clone(),
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
                EditVariableFocus::Key => Some(k),
                EditVariableFocus::Value => Some(v),
            }
        } else {
            None
        }
    }

    pub fn get_focused_variable_input_ref(&self) -> Option<&Input> {
        if self.selected_variable_index < self.variables.len() {
            let (k, v) = &self.variables[self.selected_variable_index];
            match self.variable_column_focus {
                EditVariableFocus::Key => Some(k),
                EditVariableFocus::Value => Some(v),
            }
        } else {
            None
        }
    }

    // --- Profiles Logic ---

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
        if self.selected_profile_index >= self.profile_scroll_offset + Self::MAX_PROFILES_HEIGHT {
            self.profile_scroll_offset =
                self.selected_profile_index + 1 - Self::MAX_PROFILES_HEIGHT;
        } else if self.selected_profile_index < self.profile_scroll_offset {
            self.profile_scroll_offset = self.selected_profile_index;
        }
    }

    // --- Popup Logic ---

    pub fn open_add_dependency_popup(&mut self, all_profiles: Vec<String>) {
        if self.focus != EditFocus::Profiles {
            return;
        }

        let mut available: Vec<String> = all_profiles
            .into_iter()
            .filter(|p| p != &self.profile_name && !self.profiles.contains(p))
            .collect();
        available.sort();

        self.select_popup.reset();
        self.select_popup.title = "Add Dependency".to_string();
        self.select_popup.options = available;
        self.select_popup.multi_select = true;
        self.show_select_popup = true;
    }

    pub fn close_popup(&mut self) {
        self.show_select_popup = false;
        self.select_popup.reset();
    }

    pub fn on_popup_input(&mut self, key: ratatui::crossterm::event::KeyEvent) {
        use ratatui::crossterm::event::KeyCode;

        match key.code {
            KeyCode::Esc => {
                self.close_popup();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_popup.select_previous();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_popup.select_next();
            }
            KeyCode::Enter => {
                let selected = self.select_popup.get_selected_items();
                for item in selected {
                    self.add_profile_dependency(item);
                }
                self.close_popup();
            }
            KeyCode::Char(' ') => {
                self.select_popup.toggle_selection();
            }
            _ => {}
        }
    }
}
