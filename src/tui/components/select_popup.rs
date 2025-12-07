use std::collections::HashSet;

#[derive(Default)]
pub struct SelectPopupComponent {
    pub title: String,
    pub options: Vec<String>,
    pub current_index: usize,
    pub scroll_offset: usize,
    pub multi_select: bool,
    pub selected_indices: HashSet<usize>,
}

impl SelectPopupComponent {
    pub fn new(title: String, options: Vec<String>) -> Self {
        Self {
            title,
            options,
            current_index: 0,
            scroll_offset: 0,
            multi_select: false,
            selected_indices: HashSet::new(),
        }
    }

    pub fn reset(&mut self) {
        self.options.clear();
        self.current_index = 0;
        self.scroll_offset = 0;
        self.selected_indices.clear();
        self.multi_select = false;
    }

    pub fn select_next(&mut self) {
        if self.options.is_empty() {
            return;
        }
        if self.current_index < self.options.len() - 1 {
            self.current_index += 1;
        } else {
            self.current_index = 0;
        }
    }

    pub fn select_previous(&mut self) {
        if self.options.is_empty() {
            return;
        }
        if self.current_index > 0 {
            self.current_index -= 1;
        } else {
            self.current_index = self.options.len() - 1;
        }
    }

    pub fn toggle_selection(&mut self) {
        if !self.multi_select {
            return;
        }
        if self.options.is_empty() {
            return;
        }

        if self.selected_indices.contains(&self.current_index) {
            self.selected_indices.remove(&self.current_index);
        } else {
            self.selected_indices.insert(self.current_index);
        }
    }

    pub fn get_selected_items(&self) -> Vec<String> {
        if self.multi_select {
            // Return all checked items
            let mut indices: Vec<_> = self.selected_indices.iter().cloned().collect();
            indices.sort();
            indices.iter().map(|&i| self.options[i].clone()).collect()
        } else {
            // Return currently highlighted item if any (single select mode)
            // Or maybe we treat single select as "enter confirms current highlight"?
            if let Some(item) = self.options.get(self.current_index) {
                vec![item.clone()]
            } else {
                Vec::new()
            }
        }
    }
}
