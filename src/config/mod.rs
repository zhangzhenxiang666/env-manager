use self::graph::{DependencyError, ProfileGraph};
use self::models::{Profile, ProfileNames};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub mod graph;
pub mod loader;
pub mod models;

pub struct AppConfig {
    profiles: HashMap<String, Profile>,
    graph: ProfileGraph,
}

impl AppConfig {
    pub fn new(profiles: HashMap<String, Profile>, graph: ProfileGraph) -> Self {
        Self { profiles, graph }
    }

    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }

    pub fn get_profile_mut(&mut self, name: &str) -> Option<&mut Profile> {
        self.profiles.get_mut(name)
    }

    pub fn add_profile(&mut self, name: String, profile: Profile) {
        self.profiles.insert(name, profile);
    }

    pub fn remove_profile(&mut self, name: &str) -> Option<Profile> {
        self.profiles.remove(name)
    }

    pub fn has_profile(&self, name: &str) -> bool {
        self.profiles.contains_key(name)
    }

    pub fn profile_names(&self) -> Vec<String> {
        self.profiles.keys().cloned().collect()
    }

    pub fn profiles_iter(&self) -> impl Iterator<Item = (&String, &Profile)> {
        self.profiles.iter()
    }

    pub fn profiles_iter_mut(&mut self) -> impl Iterator<Item = (&String, &mut Profile)> {
        self.profiles.iter_mut()
    }

    pub fn graph(&self) -> &ProfileGraph {
        &self.graph
    }

    pub fn rebuild_graph(&mut self) -> Result<(), DependencyError> {
        self.graph = ProfileGraph::build(&self.profiles)?;
        Ok(())
    }

    pub fn resolve_dependencies(&self, profile_name: &str) -> Result<Vec<String>, DependencyError> {
        self.graph.resolve_dependencies(profile_name)
    }

    pub fn find_path(&self, start: &str, end: &str) -> Option<Vec<String>> {
        self.graph.find_path(start, end)
    }

    pub fn get_parents(&self, profile_name: &str) -> Option<Vec<String>> {
        self.graph.get_parents(profile_name)
    }

    /// Add dependency edge (more efficient than rebuild for single additions)
    pub fn add_dependency_edge(
        &mut self,
        parent: &str,
        child: &str,
    ) -> Result<(), DependencyError> {
        self.graph.add_dependency(parent, child)
    }

    /// Remove dependency edge (more efficient than rebuild for single removals)
    pub fn remove_dependency_edge(
        &mut self,
        parent: &str,
        child: &str,
    ) -> Result<(), DependencyError> {
        self.graph.remove_dependency(parent, child)
    }

    /// Add a new profile node to graph
    pub fn add_profile_node(&mut self, name: String) {
        self.graph.add_node(name);
    }

    /// Remove a profile node from graph
    pub fn remove_profile_node(&mut self, name: &str) -> Result<(), DependencyError> {
        self.graph.remove_node(name)
    }

    /// Rename a profile node in graph
    pub fn rename_profile_node(
        &mut self,
        old_name: &str,
        new_name: String,
    ) -> Result<(), DependencyError> {
        self.graph.rename_node(old_name, new_name)
    }
}

pub struct ConfigManager {
    app_config: AppConfig,
    base_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let home = dirs::home_dir().ok_or("Could not find home directory")?;
        let base_path = home.join(".config").join("env-manage");
        let profiles_path = base_path.join("profiles");
        fs::create_dir_all(&profiles_path)?;

        let profiles = loader::load_profiles_from_dir(&profiles_path)?;
        let graph = ProfileGraph::build(&profiles)?;

        for profile_name in profiles.keys() {
            graph.resolve_dependencies(profile_name)?;
        }

        let app_config = AppConfig::new(profiles, graph);

        Ok(Self {
            app_config,
            base_path,
        })
    }

    // === Profile query methods ===
    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.app_config.get_profile(name)
    }

    pub fn get_profile_mut(&mut self, name: &str) -> Option<&mut Profile> {
        self.app_config.get_profile_mut(name)
    }

    pub fn has_profile(&self, name: &str) -> bool {
        self.app_config.has_profile(name)
    }

    pub fn list_profile_names(&self) -> ProfileNames {
        ProfileNames(self.app_config.profile_names())
    }

    // === Profile modification methods ===
    pub fn add_profile(&mut self, name: String, profile: Profile) {
        self.app_config.add_profile(name, profile);
    }

    pub fn remove_profile(&mut self, name: &str) -> Option<Profile> {
        self.app_config.remove_profile(name)
    }

    pub fn rename_profile_in_memory(
        &mut self,
        old_name: &str,
        new_name: String,
    ) -> Option<Profile> {
        if let Some(profile) = self.app_config.remove_profile(old_name) {
            self.app_config.add_profile(new_name, profile.clone());
            Some(profile)
        } else {
            None
        }
    }

    pub fn update_profile_dependencies(
        &mut self,
        profile_name: &str,
        old_dep: &str,
        new_dep: &str,
    ) {
        if let Some(profile) = self.app_config.get_profile_mut(profile_name) {
            profile.remove_profile(old_dep);
            profile.add_profile(new_dep);
        }
    }

    // === Batch access ===
    pub fn profiles_iter(&self) -> impl Iterator<Item = (&String, &Profile)> {
        self.app_config.profiles_iter()
    }

    pub fn profiles_iter_mut(&mut self) -> impl Iterator<Item = (&String, &mut Profile)> {
        self.app_config.profiles_iter_mut()
    }

    // === Graph methods ===
    pub fn resolve_dependencies(&self, profile_name: &str) -> Result<Vec<String>, DependencyError> {
        self.app_config.resolve_dependencies(profile_name)
    }

    pub fn find_path(&self, start: &str, end: &str) -> Option<Vec<String>> {
        self.app_config.find_path(start, end)
    }

    pub fn get_parents(&self, profile_name: &str) -> Option<Vec<String>> {
        self.app_config.get_parents(profile_name)
    }

    pub fn rebuild_graph(&mut self) -> Result<(), Box<dyn Error>> {
        self.app_config.rebuild_graph()?;
        Ok(())
    }

    /// Add dependency edge incrementally (more efficient than rebuild_graph)
    /// Use this when you've already validated that the edge won't create a cycle
    pub fn add_dependency_edge(&mut self, parent: &str, child: &str) -> Result<(), Box<dyn Error>> {
        self.app_config.add_dependency_edge(parent, child)?;
        Ok(())
    }

    /// Remove dependency edge incrementally (more efficient than rebuild_graph)
    pub fn remove_dependency_edge(
        &mut self,
        parent: &str,
        child: &str,
    ) -> Result<(), Box<dyn Error>> {
        self.app_config.remove_dependency_edge(parent, child)?;
        Ok(())
    }

    /// Add a new profile node to the graph
    pub fn add_profile_node(&mut self, name: String) {
        self.app_config.add_profile_node(name);
    }

    /// Remove a profile node from the graph
    pub fn remove_profile_node(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        self.app_config.remove_profile_node(name)?;
        Ok(())
    }

    /// Rename a profile node in the graph
    pub fn rename_profile_node(
        &mut self,
        old_name: &str,
        new_name: String,
    ) -> Result<(), Box<dyn Error>> {
        self.app_config.rename_profile_node(old_name, new_name)?;
        Ok(())
    }

    // === Persistence methods ===
    pub fn write_profile(&self, name: &str, profile: &Profile) -> Result<(), Box<dyn Error>> {
        loader::write_profile(&self.base_path, name, profile)
    }

    pub fn delete_profile_file(&self, name: &str) -> Result<(), Box<dyn Error>> {
        loader::delete_profile_file(&self.base_path, name)
    }

    pub fn rename_profile_file(
        &self,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), Box<dyn Error>> {
        loader::rename_profile_file(&self.base_path, old_name, new_name)
    }

    // === Global config ===
    pub fn read_global(&self) -> Result<Profile, Box<dyn Error>> {
        loader::read_global_config(&self.base_path)
    }

    pub fn write_global(&self, global: &Profile) -> Result<(), Box<dyn Error>> {
        loader::write_global_config(&self.base_path, global)
    }

    // === Internal access ===
    pub fn base_path(&self) -> &std::path::Path {
        &self.base_path
    }
}
