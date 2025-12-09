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
    fn new(profiles: HashMap<String, Profile>, graph: ProfileGraph) -> Self {
        Self { profiles, graph }
    }

    fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }

    fn get_profile_mut(&mut self, name: &str) -> Option<&mut Profile> {
        self.profiles.get_mut(name)
    }

    fn add_profile(&mut self, name: String, profile: Profile) {
        self.profiles.insert(name, profile);
    }

    fn remove_profile(&mut self, name: &str) -> Option<Profile> {
        self.profiles.remove(name)
    }

    fn has_profile(&self, name: &str) -> bool {
        self.profiles.contains_key(name)
    }

    fn profile_names(&self) -> Vec<String> {
        self.profiles.keys().cloned().collect()
    }

    fn profiles_iter(&self) -> impl Iterator<Item = (&String, &Profile)> {
        self.profiles.iter()
    }

    fn profiles_iter_mut(&mut self) -> impl Iterator<Item = (&String, &mut Profile)> {
        self.profiles.iter_mut()
    }

    fn rebuild_graph(&mut self) -> Result<(), DependencyError> {
        self.graph = ProfileGraph::build(&self.profiles)?;
        Ok(())
    }

    fn resolve_dependencies(&self, profile_name: &str) -> Result<Vec<String>, DependencyError> {
        self.graph.resolve_dependencies(profile_name)
    }

    fn find_path(&self, start: &str, end: &str) -> Option<Vec<String>> {
        self.graph.find_path(start, end)
    }

    fn get_parents(&self, profile_name: &str) -> Option<Vec<String>> {
        self.graph.get_parents(profile_name)
    }

    /// Add dependency edge (more efficient than rebuild for single additions)
    fn add_dependency_edge(&mut self, parent: &str, child: &str) -> Result<(), DependencyError> {
        self.graph.add_dependency(parent, child)
    }

    /// Remove dependency edge (more efficient than rebuild for single removals)
    fn remove_dependency_edge(&mut self, parent: &str, child: &str) -> Result<(), DependencyError> {
        self.graph.remove_dependency(parent, child)
    }

    /// Add a new profile node to graph
    fn add_profile_node(&mut self, name: String) {
        self.graph.add_node(name);
    }

    /// Remove a profile node from graph
    fn remove_profile_node(&mut self, name: &str) -> Result<(), DependencyError> {
        self.graph.remove_node(name)
    }

    /// Rename a profile node in graph
    fn rename_profile_node(
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

        // Lazy load: Start with empty profiles and graph
        let profiles = HashMap::new();
        let graph = ProfileGraph::new();
        let app_config = AppConfig::new(profiles, graph);

        Ok(Self {
            app_config,
            base_path,
        })
    }

    /// Creates a ConfigManager and loads all profiles immediately.
    /// This restores the original behavior where all profiles are loaded at startup.
    pub fn new_full() -> Result<Self, Box<dyn Error>> {
        let mut manager = Self::new()?;
        manager.load_all_profiles()?;
        Ok(manager)
    }

    pub fn load_profile(&mut self, name: &str) -> Result<(), DependencyError> {
        self.load_profile_recursive(name, &mut std::collections::HashSet::new())
    }

    fn load_profile_recursive(
        &mut self,
        name: &str,
        visiting: &mut std::collections::HashSet<String>,
    ) -> Result<(), DependencyError> {
        if self.app_config.has_profile(name) {
            return Ok(());
        }

        if visiting.contains(name) {
            return Ok(());
        }

        visiting.insert(name.to_string());

        // Load from file
        let profile = loader::load_profile_from_file(&self.base_path, name).map_err(|_| {
            // Mapping IO error (or custom error from loader) to DependencyError
            DependencyError::ProfileNotFound(name.to_string())
        })?;

        // Ensure node exists in graph
        self.app_config.add_profile_node(name.to_string());

        let mut errors = Vec::new();

        // Load dependencies
        for dep_name in &profile.profiles {
            if let Err(e) = self.load_profile_recursive(dep_name, visiting) {
                errors.push(DependencyError::DependencyChain {
                    profile: name.to_string(),
                    cause: Box::new(e),
                });
            } else {
                // Add dependency edge only if load succeeded (or cycle check passed)
                // If load failed, adding edge might cause noise or be impossible if node missing.
                if let Err(e) = self.app_config.add_dependency_edge(name, dep_name) {
                    errors.push(e);
                }
            }
        }

        if !errors.is_empty() {
            visiting.remove(name);
            if errors.len() == 1 {
                return Err(errors.pop().unwrap());
            }
            return Err(DependencyError::MultipleErrors(errors));
        }

        self.app_config.add_profile(name.to_string(), profile);
        visiting.remove(name);
        Ok(())
    }

    pub fn load_all_profiles(&mut self) -> Result<(), Box<dyn Error>> {
        let names = self.scan_profile_names()?;
        for name in names.iter() {
            self.load_profile(name)?;
        }
        Ok(())
    }

    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.app_config.get_profile(name)
    }

    pub fn get_profile_mut(&mut self, name: &str) -> Option<&mut Profile> {
        self.app_config.get_profile_mut(name)
    }

    pub fn has_profile(&self, name: &str) -> bool {
        self.app_config.has_profile(name)
    }

    /// Unlike `scan_profile_names`, it simply returns the set of profile names that are currently loaded
    pub fn list_profile_names(&self) -> ProfileNames {
        let names = self.app_config.profile_names();
        ProfileNames(names)
    }

    pub fn scan_profile_names(&self) -> Result<ProfileNames, Box<dyn Error>> {
        let names = loader::scan_profile_names(&self.base_path.join("profiles"))?;
        Ok(ProfileNames(names))
    }

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

    pub fn profiles_iter(&self) -> impl Iterator<Item = (&String, &Profile)> {
        self.app_config.profiles_iter()
    }

    pub fn profiles_iter_mut(&mut self) -> impl Iterator<Item = (&String, &mut Profile)> {
        self.app_config.profiles_iter_mut()
    }

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

    pub fn read_global(&self) -> Result<Profile, Box<dyn Error>> {
        loader::read_global_config(&self.base_path)
    }

    pub fn write_global(&self, global: &Profile) -> Result<(), Box<dyn Error>> {
        loader::write_global_config(&self.base_path, global)
    }

    pub fn base_path(&self) -> &std::path::Path {
        &self.base_path
    }
}
