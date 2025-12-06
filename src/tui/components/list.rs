use std::collections::HashSet;

#[derive(Default)]
pub struct ListComponent {
    pub profile_names: Vec<String>,
    pub selected_index: usize,
    pub dirty_profiles: HashSet<String>,
}

impl ListComponent {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn next(&mut self) {
        if self.profile_names.is_empty() {
            self.selected_index = 0;
            return;
        }
        let i = (self.selected_index + 1) % self.profile_names.len();
        self.selected_index = i;
    }

    pub fn previous(&mut self) {
        if self.profile_names.is_empty() {
            self.selected_index = 0;
            return;
        }
        let i = (self.selected_index + self.profile_names.len() - 1) % self.profile_names.len();
        self.selected_index = i;
    }
}
