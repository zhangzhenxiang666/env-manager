use crate::GLOBAL_PROFILE_MARK;
use crate::tui::utils::Input;
use std::collections::HashSet;

#[derive(Default)]
pub struct ListComponent {
    profile_names: Vec<String>,
    selected_index: usize,
    dirty_profiles: HashSet<String>,
    rename_input: Input,
    in_search_mode: bool,
    search_input: Input,
}

impl ListComponent {
    pub fn new() -> Self {
        Default::default()
    }

    /// Get the currently selected profile name
    pub fn current_profile(&self) -> Option<&str> {
        self.profile_names
            .get(self.selected_index)
            .map(|s| s.as_str())
    }

    /// Get all profile names (unfiltered)
    pub fn all_profiles(&self) -> &[String] {
        &self.profile_names
    }

    /// Get filtered profiles based on search mode
    pub fn filtered_profiles(&self) -> Vec<String> {
        if !self.in_search_mode || self.search_input.text().is_empty() {
            return self.profile_names.clone();
        }

        let search_query = self.search_input.text().to_lowercase();
        self.profile_names
            .iter()
            .filter(|name| name.to_lowercase().contains(&search_query))
            .cloned()
            .collect()
    }

    /// Update the profile list (e.g., after adding/removing profiles)
    pub fn update_profiles(&mut self, mut profiles: Vec<String>) {
        profiles.sort_by(|a, b| {
            if a == GLOBAL_PROFILE_MARK {
                std::cmp::Ordering::Less
            } else if b == GLOBAL_PROFILE_MARK {
                std::cmp::Ordering::Greater
            } else {
                a.cmp(b)
            }
        });
        self.profile_names = profiles;
        // Ensure selected_index is valid
        if self.selected_index >= self.profile_names.len() && !self.profile_names.is_empty() {
            self.selected_index = self.profile_names.len() - 1;
        } else if self.profile_names.is_empty() {
            self.selected_index = 0;
        }
    }

    /// Get current selected index (for rendering)
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Set selected index directly (for after operations that change list)
    pub fn set_selected_index(&mut self, index: usize) {
        if index < self.profile_names.len() {
            self.selected_index = index;
        }
    }

    pub fn next(&mut self) {
        let filtered = self.filtered_profiles();
        if filtered.is_empty() {
            self.selected_index = 0;
            return;
        }
        let i = (self.selected_index + 1) % filtered.len();
        self.selected_index = i;
    }

    pub fn previous(&mut self) {
        let filtered = self.filtered_profiles();
        if filtered.is_empty() {
            self.selected_index = 0;
            return;
        }
        let i = (self.selected_index + filtered.len() - 1) % filtered.len();
        self.selected_index = i;
    }

    /// Check if a specific profile has unsaved changes
    pub fn is_dirty(&self, name: &str) -> bool {
        self.dirty_profiles.contains(name)
    }

    /// Get count of profiles with unsaved changes
    pub fn unsaved_count(&self) -> usize {
        self.dirty_profiles.len()
    }

    /// Mark a profile as having unsaved changes
    pub fn mark_dirty(&mut self, name: String) {
        self.dirty_profiles.insert(name);
    }

    /// Clear dirty flag for a profile (after saving)
    pub fn clear_dirty(&mut self, name: &str) {
        self.dirty_profiles.remove(name);
    }

    /// Get iterator over all dirty profile names
    pub fn dirty_profiles_iter(&self) -> impl Iterator<Item = &String> {
        self.dirty_profiles.iter()
    }

    pub fn is_searching(&self) -> bool {
        self.in_search_mode
    }

    pub fn enter_search_mode(&mut self) {
        self.in_search_mode = true;
        self.search_input.reset();
        self.selected_index = 0;
    }

    pub fn exit_search_mode(&mut self) {
        if !self.in_search_mode {
            return;
        }
        let filtered = self.filtered_profiles();
        if !filtered.is_empty() {
            let selected_name = &filtered[self.selected_index];
            if let Some(index) = self
                .profile_names
                .iter()
                .position(|name| name == selected_name)
            {
                self.selected_index = index;
            }
        }
        self.in_search_mode = false;
        self.search_input.reset();
    }

    /// Get mutable reference to search input for event handlers
    pub fn search_input_mut(&mut self) -> &mut Input {
        &mut self.search_input
    }

    /// Get reference to search input for rendering
    pub fn search_input(&self) -> &Input {
        &self.search_input
    }

    pub fn start_rename(&mut self) {
        if let Some(current_name) = self.current_profile() {
            let name = current_name.to_string();
            self.rename_input.set_text(name.clone());
            self.rename_input.set_cursor_position(name.len());
            self.rename_input.clear_error();
        }
    }

    /// Get mutable reference to rename input for event handlers
    pub fn rename_input_mut(&mut self) -> &mut Input {
        &mut self.rename_input
    }

    /// Get reference to rename input for rendering
    pub fn rename_input(&self) -> &Input {
        &self.rename_input
    }

    /// Reset rename input
    pub fn reset_rename(&mut self) {
        self.rename_input.reset();
    }
}
