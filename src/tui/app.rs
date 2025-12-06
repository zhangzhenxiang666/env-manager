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

        // Delete file
        loader::delete_profile_file(&self.config_manager.base_path, &name_to_delete)?;

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
