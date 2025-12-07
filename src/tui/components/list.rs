use std::collections::HashSet;

use crate::tui::utils::Input;

#[derive(Default)]
pub struct ListComponent {
    pub profile_names: Vec<String>,
    pub selected_index: usize,
    pub dirty_profiles: HashSet<String>,
    pub rename_input: Input,
    pub in_search_mode: bool,
    pub search_input: Input,
}

impl ListComponent {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_filtered_profiles(&self) -> Vec<String> {
        if !self.in_search_mode || self.search_input.text.is_empty() {
            return self.profile_names.clone();
        }

        let search_query = self.search_input.text.to_lowercase();
        self.profile_names
            .iter()
            .filter(|name| name.to_lowercase().contains(&search_query))
            .cloned()
            .collect()
    }

    pub fn next(&mut self) {
        let filtered = self.get_filtered_profiles();
        if filtered.is_empty() {
            self.selected_index = 0;
            return;
        }
        let i = (self.selected_index + 1) % filtered.len();
        self.selected_index = i;
    }

    pub fn previous(&mut self) {
        let filtered = self.get_filtered_profiles();
        if filtered.is_empty() {
            self.selected_index = 0;
            return;
        }
        let i = (self.selected_index + filtered.len() - 1) % filtered.len();
        self.selected_index = i;
    }

    pub fn exit_search_mode(&mut self) {
        if !self.in_search_mode {
            return;
        }
        let filtered = self.get_filtered_profiles();
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
}
