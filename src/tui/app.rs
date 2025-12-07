use super::event::handle_event;
use super::ui::ui;
use crate::config::ConfigManager;
use crate::config::graph::ProfileGraph;
use crate::config::loader;
use crate::tui::components::add_new::AddNewComponent;
use crate::tui::components::list::ListComponent;
use daggy::Walker;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::prelude::Backend;
use ratatui::{Terminal, prelude::CrosstermBackend};

use std::io;

#[derive(Default, Debug, PartialEq, Eq)]
pub enum AppState {
    #[default]
    List,
    Edit,
    AddNew,
    Rename,
    ConfirmDelete,
    Search,
}

pub struct App {
    pub config_manager: ConfigManager,
    pub state: AppState,
    pub shutdown: bool,
    pub add_new_component: AddNewComponent,
    pub list_component: ListComponent,
    pub status_message: Option<String>,
    pub pending_deletes: std::collections::HashMap<String, Option<String>>,
}

impl App {
    pub fn new(config_manager: ConfigManager) -> App {
        let mut profile_names: Vec<String> =
            config_manager.app_config.profiles.keys().cloned().collect();
        profile_names.sort();

        let mut list_component = ListComponent::new();
        list_component.profile_names = profile_names;

        App {
            config_manager,
            state: Default::default(),
            shutdown: false,
            add_new_component: Default::default(),
            list_component,
            status_message: None,
            pending_deletes: std::collections::HashMap::new(),
        }
    }

    pub fn next(&mut self) {
        self.list_component.next();
    }

    pub fn previous(&mut self) {
        self.list_component.previous();
    }

    pub fn save_selected(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.list_component.profile_names.is_empty() {
            return Ok(());
        }
        let name = &self.list_component.profile_names[self.list_component.selected_index];
        if self.list_component.dirty_profiles.contains(name) {
            if let Some(profile) = self.config_manager.app_config.profiles.get(name) {
                loader::write_profile(&self.config_manager.base_path, name, profile)?;
                self.list_component.dirty_profiles.remove(name);
            }
        }

        // Recursive delete logic for rename chains
        // Find if the current saved name is a target of a rename
        // i.e., find 'old_name' where map[old_name] == Some(name)
        let mut to_delete = Vec::new();

        // simple linear search is fine for small number of pending deletes
        for (old, new_opt) in self.pending_deletes.iter() {
            if let Some(new_name) = new_opt {
                if new_name == name {
                    to_delete.push(old.clone());
                }
            }
        }

        // For each found predecessor, delete it, and recursively check for its predecessor
        let mut queue = to_delete;
        while let Some(del_name) = queue.pop() {
            if self.pending_deletes.contains_key(&del_name) {
                // Execute delete
                // Check if it still exists on disk before errors? loader::delete handle it usually.
                // We perform delete
                loader::delete_profile_file(&self.config_manager.base_path, &del_name)?;

                // Remove from pending map
                self.pending_deletes.remove(&del_name);

                // Check who pointed to 'del_name'
                for (old, new_opt) in self.pending_deletes.iter() {
                    if let Some(new_name) = new_opt {
                        if new_name == &del_name {
                            queue.push(old.clone());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn save_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Clone the set to avoid borrowing issues while iterating and modifying
        let dirty_names: Vec<String> = self.list_component.dirty_profiles.iter().cloned().collect();
        for name in dirty_names {
            if let Some(profile) = self.config_manager.app_config.profiles.get(&name) {
                loader::write_profile(&self.config_manager.base_path, &name, profile)?;
                self.list_component.dirty_profiles.remove(&name);
            }
        }

        // Process all pending deletes
        // We can just iterate keys and delete
        let all_deletes: Vec<String> = self.pending_deletes.keys().cloned().collect();
        for name in all_deletes {
            loader::delete_profile_file(&self.config_manager.base_path, &name)?;
            self.pending_deletes.remove(&name);
        }

        Ok(())
    }

    pub fn rename_profile(&mut self, new_name: String) -> Result<(), Box<dyn std::error::Error>> {
        if self.list_component.profile_names.is_empty() {
            return Ok(());
        }
        let old_name =
            self.list_component.profile_names[self.list_component.selected_index].clone();

        if old_name == new_name {
            return Ok(());
        }

        // 1. Update Profile Map
        if let Some(profile) = self.config_manager.app_config.profiles.remove(&old_name) {
            self.config_manager
                .app_config
                .profiles
                .insert(new_name.clone(), profile);
        } else {
            return Err(format!("Profile '{old_name}' not found in memory.").into());
        }

        // 2. Queue old name for deletion (Linked to new name)
        self.pending_deletes
            .insert(old_name.clone(), Some(new_name.clone()));

        // 3. Update Dependencies (other profiles that use old_name)
        let mut affected_profiles = Vec::new();
        for (name, profile) in self.config_manager.app_config.profiles.iter_mut() {
            if profile.profiles.contains(&old_name) {
                profile.profiles.remove(&old_name);
                profile.profiles.insert(new_name.clone());
                affected_profiles.push(name.clone());
            }
        }

        // 4. Mark affected profiles as dirty
        for name in affected_profiles {
            self.list_component.dirty_profiles.insert(name);
        }

        // 5. Mark new profile as dirty (it has a new name/location essentially)
        self.list_component.dirty_profiles.insert(new_name.clone());
        // Since we removed old_name, remove it from dirty if it was there
        self.list_component.dirty_profiles.remove(&old_name);

        // 6. Rebuild Graph
        self.config_manager.app_config.graph =
            ProfileGraph::build(&self.config_manager.app_config.profiles)?;

        // 7. Update List Component
        self.list_component.profile_names[self.list_component.selected_index] = new_name.clone();
        // Resort list
        self.list_component.profile_names.sort();
        // Fix selected index to follow the renamed item
        if let Some(new_index) = self
            .list_component
            .profile_names
            .iter()
            .position(|n| n == &new_name)
        {
            self.list_component.selected_index = new_index;
        }

        self.status_message = Some(format!("Renamed '{old_name}' to '{new_name}'"));
        Ok(())
    }

    pub fn delete_selected_profile(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.list_component.profile_names.is_empty() {
            return Ok(());
        }

        let name_to_delete =
            self.list_component.profile_names[self.list_component.selected_index].clone();

        // Validation
        let graph = &self.config_manager.app_config.graph;
        if let Some(&node_index) = graph.profile_nodes.get(&name_to_delete) {
            let walker = graph.graph.parents(node_index);
            let dependents: Vec<String> = walker
                .iter(&graph.graph)
                .map(|(_edge, node)| graph.graph[node].clone())
                .collect();

            if !dependents.is_empty() {
                let error_message = format!(
                    "Cannot delete '{}' as it is used by: {}",
                    name_to_delete,
                    dependents.join(", ")
                );
                self.status_message = Some(error_message);
                return Ok(());
            }
        }

        self.list_component
            .profile_names
            .remove(self.list_component.selected_index);

        // Queue for deletion (No successor)
        self.pending_deletes.insert(name_to_delete.clone(), None);
        // Also: Immediate deletion from disk?
        // Logic says: User pressed delete, it should probably be gone?
        // BUT: if we want "undo" or consistent "save to apply", we might wait.
        // HOWEVER: The prompts says "Safe Delete", usually implies immediate effect or confirmed.
        // Existing code did: loader::delete_profile_file immediately.
        // If we want to keep existing behavior + consistency:
        loader::delete_profile_file(&self.config_manager.base_path, &name_to_delete)?;
        self.pending_deletes.remove(&name_to_delete); // Done.

        // Remove from config manager's in-memory cache
        self.config_manager
            .app_config
            .profiles
            .remove(&name_to_delete);

        // Remove from dirty set if it's there
        self.list_component.dirty_profiles.remove(&name_to_delete);

        // Rebuild graph
        self.config_manager.app_config.graph =
            ProfileGraph::build(&self.config_manager.app_config.profiles)?;

        // Adjust selected index
        if !self.list_component.profile_names.is_empty()
            && self.list_component.selected_index >= self.list_component.profile_names.len()
        {
            self.list_component.selected_index = self.list_component.profile_names.len() - 1;
        } else if self.list_component.profile_names.is_empty() {
            self.list_component.selected_index = 0;
        }

        self.status_message = Some(format!("Successfully deleted '{name_to_delete}'"));
        Ok(())
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let mut app = App::new(ConfigManager::new()?);

        enable_raw_mode()?;
        let mut stderr = io::stderr();
        execute!(stderr, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stderr);
        let mut terminal = Terminal::new(backend)?;

        let res = run_app(&mut terminal, &mut app);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        res
    }
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        if app.shutdown {
            return Ok(());
        }

        terminal.draw(|frame| ui(frame, app))?;

        handle_event(app)?;
    }
}
