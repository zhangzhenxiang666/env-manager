use super::components::add_new::AddNewComponent;
use super::components::edit::EditComponent;
use super::components::list::ListComponent;
use super::event::handle_event;
use super::ui::ui;
use crate::GLOBAL_PROFILE_MARK;
use crate::config::ConfigManager;
use crate::config::models::Profile;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::prelude::Backend;
use ratatui::{Terminal, prelude::CrosstermBackend};
use std::collections::HashMap;
use std::io;

#[derive(Default, Debug, PartialEq, Eq)]
pub enum AppState {
    #[default]
    List,
    Edit,
    AddNew,
    Rename,
    ConfirmDelete,
    ConfirmExit,
}

#[derive(Default, PartialEq, Eq)]
pub enum MainRightViewMode {
    #[default]
    Raw,
    Expand,
}

pub struct App {
    pub config_manager: ConfigManager,
    pub state: AppState,
    pub shutdown: bool,
    pub add_new_component: AddNewComponent,
    pub edit_component: EditComponent,
    pub main_right_view_mode: MainRightViewMode,
    pub expand_env_vars: Option<HashMap<String, String>>,
    pub list_component: ListComponent,
    pub status_message: Option<String>,
    pub pending_deletes: HashMap<String, Option<String>>,
}

impl App {
    pub fn new(mut config_manager: ConfigManager, global_profile: Profile) -> App {
        // Load GLOBAL profile
        config_manager.add_profile(GLOBAL_PROFILE_MARK.to_string(), global_profile);

        let mut list_component = ListComponent::new();
        let profile_names: Vec<String> = config_manager.list_profile_names().to_vec();
        list_component.update_profiles(profile_names);

        App {
            config_manager,
            state: Default::default(),
            shutdown: false,
            add_new_component: Default::default(),
            edit_component: Default::default(),
            list_component,
            status_message: None,
            pending_deletes: Default::default(),
            main_right_view_mode: Default::default(),
            expand_env_vars: Default::default(),
        }
    }

    pub fn save_selected(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let name = match self.list_component.current_profile() {
            Some(n) => n.to_string(),
            None => return Ok(()),
        };

        if self.list_component.is_dirty(&name)
            && let Some(profile) = self.config_manager.get_profile(&name)
        {
            self.config_manager.write_profile(&name, profile)?;
            self.list_component.clear_dirty(&name);
        }

        // Recursive delete logic for rename chains
        // Find if the current saved name is a target of a rename
        // i.e., find 'old_name' where map[old_name] == Some(name)
        let mut to_delete = Vec::new();

        // simple linear search is fine for small number of pending deletes
        for (old, new_opt) in self.pending_deletes.iter() {
            if let Some(new_name) = new_opt
                && new_name == &name
            {
                to_delete.push(old.clone());
            }
        }

        // For each found predecessor, delete it, and recursively check for its predecessor
        let mut queue = to_delete;
        while let Some(del_name) = queue.pop() {
            if self.pending_deletes.contains_key(&del_name) {
                self.config_manager.delete_profile_file(&del_name)?;

                // Remove from pending map
                self.pending_deletes.remove(&del_name);

                // Check who pointed to 'del_name'
                for (old, new_opt) in self.pending_deletes.iter() {
                    if let Some(new_name) = new_opt
                        && new_name == &del_name
                    {
                        queue.push(old.clone());
                    }
                }
            }
        }

        Ok(())
    }

    pub fn save_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let dirty_names: Vec<String> = self.list_component.dirty_profiles_iter().cloned().collect();
        for name in dirty_names {
            if let Some(profile) = self.config_manager.get_profile(&name) {
                self.config_manager.write_profile(&name, profile)?;
                self.list_component.clear_dirty(&name);
            }
        }

        // Process all pending deletes
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

        if old_name == GLOBAL_PROFILE_MARK {
            return Ok(());
        }

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

    pub fn load_expand_vars(&mut self) {
        if let Some(profile_name) = self.list_component.current_profile()
            && let Some(profile) = self.config_manager.get_profile(profile_name)
        {
            match profile.collect_vars(&self.config_manager) {
                Ok(vars) => {
                    self.expand_env_vars = Some(vars);
                    self.main_right_view_mode = MainRightViewMode::Expand;
                }
                Err(e) => {
                    self.status_message = Some(format!("Error expanding variables: {e}"));
                }
            }
        }
    }

    pub fn unload_expand_vars(&mut self) {
        self.expand_env_vars.take();
        self.main_right_view_mode = MainRightViewMode::Raw;
    }

    pub fn delete_selected_profile(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let name_to_delete = match self.list_component.current_profile() {
            Some(n) => n.to_string(),
            None => return Ok(()),
        };

        // Validation
        if name_to_delete == GLOBAL_PROFILE_MARK {
            return Ok(());
        }

        if let Some(dependents) = self.config_manager.get_parents(&name_to_delete)
            && !dependents.is_empty()
        {
            let error_message = format!(
                "Cannot delete '{}' as it is used by: {}",
                name_to_delete,
                dependents.join(", ")
            );
            self.status_message = Some(error_message);
            return Ok(());
        }

        let mut profiles = self.list_component.all_profiles().to_vec();
        let selected_idx = self.list_component.selected_index();
        if selected_idx < profiles.len() {
            profiles.remove(selected_idx);
        }
        self.list_component.update_profiles(profiles);

        self.config_manager.delete_profile_file(&name_to_delete)?;

        // Remove from config manager's in-memory cache
        self.config_manager.remove_profile(&name_to_delete);

        // Remove from dirty set if it's there
        self.list_component.clear_dirty(&name_to_delete);

        // Remove from graph incrementally (more efficient than rebuild)
        self.config_manager.remove_profile_node(&name_to_delete)?;

        self.status_message = Some(format!("Successfully deleted '{name_to_delete}'"));

        Ok(())
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let config_manager = ConfigManager::new_full()?;
        let global_profile = config_manager.read_global()?;
        let mut app = App::new(config_manager, global_profile);

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
