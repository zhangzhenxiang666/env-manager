use super::event::handle_event;
use super::ui::ui;
use crate::config::ConfigManager;
use crate::tui::components::add_new::AddNewComponent;
use crate::tui::components::edit::EditComponent;
use crate::tui::components::list::ListComponent;
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
}

pub struct App {
    pub config_manager: ConfigManager,
    pub state: AppState,
    pub shutdown: bool,
    pub add_new_component: AddNewComponent,
    pub edit_component: EditComponent,
    pub list_component: ListComponent,
    pub status_message: Option<String>,
    pub pending_deletes: std::collections::HashMap<String, Option<String>>,
}

impl App {
    pub fn new(config_manager: ConfigManager) -> App {
        let mut profile_names: Vec<String> = config_manager.list_profile_names().to_vec();
        profile_names.sort();
        profile_names.sort();

        let mut list_component = ListComponent::new();
        list_component.update_profiles(profile_names);

        App {
            config_manager,
            state: Default::default(),
            shutdown: false,
            add_new_component: Default::default(),
            edit_component: Default::default(),
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
        let name = match self.list_component.current_profile() {
            Some(n) => n.to_string(),
            None => return Ok(()),
        };

        if self.list_component.is_dirty(&name) {
            if let Some(profile) = self.config_manager.get_profile(&name) {
                self.config_manager.write_profile(&name, profile)?;
                self.list_component.clear_dirty(&name);
            }
        }

        // Recursive delete logic for rename chains
        // Find if the current saved name is a target of a rename
        // i.e., find 'old_name' where map[old_name] == Some(name)
        let mut to_delete = Vec::new();

        // simple linear search is fine for small number of pending deletes
        for (old, new_opt) in self.pending_deletes.iter() {
            if let Some(new_name) = new_opt {
                if new_name == &name {
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
                self.config_manager.delete_profile_file(&del_name)?;

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
        let dirty_names: Vec<String> = self.list_component.dirty_profiles_iter().cloned().collect();
        for name in dirty_names {
            if let Some(profile) = self.config_manager.get_profile(&name) {
                self.config_manager.write_profile(&name, profile)?;
                self.list_component.clear_dirty(&name);
            }
        }

        // Process all pending deletes
        // We can just iterate keys and delete
        let all_deletes: Vec<String> = self.pending_deletes.keys().cloned().collect();
        for name in all_deletes {
            self.config_manager.delete_profile_file(&name)?;
            self.pending_deletes.remove(&name);
        }

        Ok(())
    }

    pub fn rename_profile(&mut self, new_name: String) -> Result<(), Box<dyn std::error::Error>> {
        let old_name = match self.list_component.current_profile() {
            Some(n) => n.to_string(),
            None => return Ok(()),
        };

        if old_name == new_name {
            return Ok(());
        }

        // 1. Update Profile Map
        if let Some(profile) = self.config_manager.remove_profile(&old_name) {
            self.config_manager.add_profile(new_name.clone(), profile);
        } else {
            return Err(format!("Profile '{old_name}' not found in memory.").into());
        }

        // 2. Queue old name for deletion (Linked to new name)
        self.pending_deletes
            .insert(old_name.clone(), Some(new_name.clone()));

        // 3. Update Dependencies (other profiles that use old_name)
        let mut affected_profiles = Vec::new();
        for (name, profile) in self.config_manager.profiles_iter_mut() {
            if profile.profiles.contains(&old_name) {
                profile.profiles.remove(&old_name);
                profile.profiles.insert(new_name.clone());
                affected_profiles.push(name.clone());
            }
        }

        // 4. Mark affected profiles as dirty
        for name in affected_profiles {
            self.list_component.mark_dirty(name);
        }

        // 5. Mark new profile as dirty (it has a new name/location essentially)
        self.list_component.mark_dirty(new_name.clone());
        // Since we removed old_name, remove it from dirty if it was there
        self.list_component.clear_dirty(&old_name);

        // 6. Update graph incrementally (more efficient than rebuild)
        self.config_manager
            .rename_profile_node(&old_name, new_name.clone())?;

        // 7. Update List Component
        let mut profiles = self.list_component.all_profiles().to_vec();
        if let Some(pos) = profiles.iter().position(|n| n == &old_name) {
            profiles[pos] = new_name.clone();
        }
        profiles.sort();
        self.list_component.update_profiles(profiles);

        // Fix selected index to follow the renamed item
        if let Some(new_index) = self
            .list_component
            .all_profiles()
            .iter()
            .position(|n| n == &new_name)
        {
            self.list_component.set_selected_index(new_index);
        }

        self.status_message = Some(format!("Renamed '{old_name}' to '{new_name}'"));
        Ok(())
    }

    pub fn start_editing(&mut self, profile_name: &str) {
        if let Some(profile) = self.config_manager.get_profile(profile_name) {
            self.edit_component = EditComponent::from_profile(profile_name, profile);
            self.state = AppState::Edit;
        }
    }

    pub fn delete_selected_profile(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let name_to_delete = match self.list_component.current_profile() {
            Some(n) => n.to_string(),
            None => return Ok(()),
        };

        // Validation
        if let Some(dependents) = self.config_manager.get_parents(&name_to_delete) {
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

        let mut profiles = self.list_component.all_profiles().to_vec();
        let selected_idx = self.list_component.selected_index();
        if selected_idx < profiles.len() {
            profiles.remove(selected_idx);
        }
        self.list_component.update_profiles(profiles);

        // Queue for deletion (No successor)
        self.pending_deletes.insert(name_to_delete.clone(), None);
        // Also: Immediate deletion from disk?
        // Logic says: User pressed delete, it should probably be gone?
        // BUT: if we want "undo" or consistent "save to apply", we might wait.
        // HOWEVER: The prompts says "Safe Delete", usually implies immediate effect or confirmed.
        // Existing code did: loader::delete_profile_file immediately.
        // If we want to keep existing behavior + consistency:
        self.config_manager.delete_profile_file(&name_to_delete)?;
        self.pending_deletes.remove(&name_to_delete); // Done.

        // Remove from config manager's in-memory cache
        self.config_manager.remove_profile(&name_to_delete);

        // Remove from dirty set if it's there
        self.list_component.clear_dirty(&name_to_delete);

        // Remove from graph incrementally (more efficient than rebuild)
        self.config_manager.remove_profile_node(&name_to_delete)?;

        // Adjust selected index (update_profiles handles this now, but let's be explicit)
        let profile_count = self.list_component.all_profiles().len();
        let current_idx = self.list_component.selected_index();
        if profile_count > 0 && current_idx >= profile_count {
            self.list_component.set_selected_index(profile_count - 1);
        } else if profile_count == 0 {
            self.list_component.set_selected_index(0);
        }

        self.status_message = Some(format!("Successfully deleted '{name_to_delete}'"));
        Ok(())
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let mut app = App::new(ConfigManager::new_full()?);

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
