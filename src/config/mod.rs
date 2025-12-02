use self::models::{Profile, ProfileNames};
use daggy::{Dag, NodeIndex, Walker};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::PathBuf;

pub mod models;

// Custom error for dependency resolution
#[derive(Debug)]
pub enum DependencyError {
    CircularDependency(Vec<String>),
    ProfileNotFound(String),
}

impl fmt::Display for DependencyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DependencyError::CircularDependency(path) => {
                write!(f, "Circular dependency detected: {}", path.join(" -> "))
            }
            DependencyError::ProfileNotFound(name) => {
                write!(f, "Profile '{name}' not found in configuration.")
            }
        }
    }
}

impl Error for DependencyError {}

pub struct AppConfig {
    pub profiles: HashMap<String, Profile>,
    pub graph: Dag<String, ()>,
    pub profile_nodes: HashMap<String, NodeIndex>,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
            graph: Dag::new(),
            profile_nodes: HashMap::new(),
        }
    }
}

/// Manages reading from and writing to the application's configuration files.
pub struct ConfigManager {
    pub app_config: AppConfig,
    base_path: PathBuf,
}

impl ConfigManager {
    /// Creates a new instance of the `ConfigManager`.
    /// This will also ensure that the configuration directory exists, load all profiles,
    /// and validate the dependency graph for cycles.
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let home = dirs::home_dir().ok_or("Could not find home directory")?;
        let base_path = home.join(".config").join("env-manage");
        let profiles_path = base_path.join("profiles");
        fs::create_dir_all(&profiles_path)?;

        let mut app_config = AppConfig::new();

        for entry in fs::read_dir(&profiles_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Some(profile_name) = path.file_stem().and_then(|s| s.to_str()) {
                    let content = fs::read_to_string(&path)?;
                    let profile: Profile = toml::from_str(&content)?;
                    app_config
                        .profiles
                        .insert(profile_name.to_string(), profile);
                }
            }
        }

        for name in app_config.profiles.keys() {
            let index = app_config.graph.add_node(name.clone());
            app_config.profile_nodes.insert(name.clone(), index);
        }

        for (name, profile) in &app_config.profiles {
            if let Some(&parent_index) = app_config.profile_nodes.get(name) {
                for dep_name in &profile.profiles {
                    if let Some(&dep_index) = app_config.profile_nodes.get(dep_name) {
                        app_config
                            .graph
                            .add_edge(parent_index, dep_index, ())
                            .map_err(|e| format!("Failed to add dependency edge: {e:?}"))?;
                    } else {
                        return Err(DependencyError::ProfileNotFound(dep_name.clone()).into());
                    }
                }
            }
        }

        let manager = Self {
            app_config,
            base_path,
        };

        // Validate the entire dependency graph on startup
        for profile_name in manager
            .app_config
            .profiles
            .keys()
            .cloned()
            .collect::<Vec<_>>()
        {
            manager.resolve_dependencies(&profile_name)?;
        }

        Ok(manager)
    }

    /// Resolves the dependency graph for a given profile name, performing a topological sort
    /// and detecting circular dependencies.
    ///
    /// Returns a flattened, ordered list of profile names, with the deepest dependencies first.
    pub fn resolve_dependencies(&self, profile_name: &str) -> Result<Vec<String>, DependencyError> {
        let mut resolved = HashSet::new();
        let mut visiting = Vec::new();
        let mut result = Vec::new();

        self.dfs_resolve(profile_name, &mut visiting, &mut resolved, &mut result)?;

        Ok(result)
    }

    /// Helper function to perform DFS for dependency resolution.
    fn dfs_resolve<'a>(
        &'a self,
        profile_name: &'a str,
        visiting: &mut Vec<&'a str>,
        resolved: &mut HashSet<&'a str>,
        result: &mut Vec<String>,
    ) -> Result<(), DependencyError> {
        visiting.push(profile_name);

        if let Some(&node_index) = self.app_config.profile_nodes.get(profile_name) {
            for (_, child_index) in self
                .app_config
                .graph
                .children(node_index)
                .iter(&self.app_config.graph)
            {
                let dep_name = &self.app_config.graph[child_index];

                if resolved.contains(dep_name.as_str()) {
                    continue;
                }
                // If the dependency is already in the current visiting path, we have a cycle.
                if let Some(pos) = visiting.iter().position(|p| p == &dep_name.as_str()) {
                    let mut cycle_path = visiting[pos..]
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>();
                    cycle_path.push(dep_name.clone());
                    return Err(DependencyError::CircularDependency(cycle_path));
                }
                self.dfs_resolve(dep_name, visiting, resolved, result)?;
            }
        } else {
            return Err(DependencyError::ProfileNotFound(profile_name.to_string()));
        }

        visiting.pop();
        if resolved.insert(profile_name) {
            result.push(profile_name.to_string());
        }

        Ok(())
    }

    /// Finds a dependency path from a starting profile to an ending profile.
    ///
    /// Returns `Some(path)` if a path is found, otherwise `None`.
    pub fn find_path(&self, start_node: &str, end_node: &str) -> Option<Vec<String>> {
        if let (Some(&start_index), Some(&end_index)) = (
            self.app_config.profile_nodes.get(start_node),
            self.app_config.profile_nodes.get(end_node),
        ) {
            let mut found_path = None;
            let mut visiting = HashSet::new();
            let mut path_stack = vec![start_index];

            self.find_path_dag(end_index, &mut path_stack, &mut visiting, &mut found_path);
            return found_path;
        }
        None
    }

    fn find_path_dag(
        &self,
        end_index: NodeIndex,
        path_stack: &mut Vec<NodeIndex>,
        visiting: &mut HashSet<NodeIndex>,
        found_path: &mut Option<Vec<String>>,
    ) {
        let current_index = *path_stack.last().unwrap();
        visiting.insert(current_index);

        for (_, child_index) in self
            .app_config
            .graph
            .children(current_index)
            .iter(&self.app_config.graph)
        {
            if found_path.is_some() {
                return;
            }

            path_stack.push(child_index);

            if child_index == end_index {
                *found_path = Some(
                    path_stack
                        .iter()
                        .map(|&i| self.app_config.graph[i].clone())
                        .collect(),
                );
                return;
            }

            if !visiting.contains(&child_index) {
                self.find_path_dag(end_index, path_stack, visiting, found_path);
            }

            path_stack.pop();
        }
    }

    /// Reads the global configuration from `global.toml`.
    ///
    /// If the file does not exist or is empty, returns a default, empty configuration.
    pub fn read_global(&self) -> Result<Profile, Box<dyn Error>> {
        let path = self.base_path.join("global.toml");
        if !path.exists() {
            return Ok(Profile::new());
        }

        let content = fs::read_to_string(path)?;
        if content.trim().is_empty() {
            return Ok(Profile::new());
        }

        Ok(toml::from_str(&content)?)
    }

    /// Saves the global configuration to `global.toml`.
    pub fn write_global(&self, global: &Profile) -> Result<(), Box<dyn Error>> {
        let path = self.base_path.join("global.toml");
        let content = toml::to_string_pretty(global)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Reads a profile by its name from the in-memory cache.
    ///
    /// Returns `Ok(None)` if the profile does not exist.
    pub fn read_profile(&self, name: &str) -> Option<&Profile> {
        self.app_config.profiles.get(name)
    }

    /// Saves a profile by its name and updates the in-memory state.
    pub fn write_profile(&mut self, name: &str, profile: &Profile) -> Result<(), Box<dyn Error>> {
        let path = self.base_path.join("profiles").join(format!("{name}.toml"));
        let content = toml::to_string_pretty(profile)?;
        fs::write(path, content)?;

        let is_new_profile = !self.app_config.profiles.contains_key(name);

        if is_new_profile {
            let node_index = self.app_config.graph.add_node(name.to_string());
            self.app_config
                .profile_nodes
                .insert(name.to_string(), node_index);
        }

        self.app_config
            .profiles
            .insert(name.to_string(), profile.clone());

        // Rebuild the edges for this node
        if let Some(&node_index) = self.app_config.profile_nodes.get(name) {
            // Remove old edges
            let old_edges: Vec<_> = self
                .app_config
                .graph
                .children(node_index)
                .iter(&self.app_config.graph)
                .map(|(edge, _)| edge)
                .collect();
            for edge in old_edges {
                self.app_config.graph.remove_edge(edge);
            }

            // Add new edges
            for dep_name in &profile.profiles {
                if let Some(&dep_index) = self.app_config.profile_nodes.get(dep_name) {
                    self.app_config
                        .graph
                        .add_edge(node_index, dep_index, ())
                        .map_err(|e| format!("Failed to add dependency edge: {e:?}"))?;
                } else {
                    // If a dependency doesn't exist, we should roll back the changes or handle it
                    // For now, let's return an error. A more robust solution might be needed.
                    return Err(DependencyError::ProfileNotFound(dep_name.clone()).into());
                }
            }
        }

        // Validate dependencies after update, to ensure no cycles were introduced
        self.resolve_dependencies(name)?;

        Ok(())
    }

    /// Retrieves the names of all profiles from memory.
    pub fn list_profile_names(&self) -> ProfileNames {
        let names = self.app_config.profiles.keys().cloned().collect();
        ProfileNames(names)
    }

    /// Checks if a profile with the given name exists in memory.
    pub fn has_profile(&self, name: &str) -> bool {
        self.app_config.profiles.contains_key(name)
    }

    /// Deletes a profile by its name and updates the in-memory state.
    pub fn delete_profile(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        let path = self.base_path.join("profiles").join(format!("{name}.toml"));
        if path.exists() {
            fs::remove_file(path)?;
        }

        self.app_config.profiles.remove(name);
        if let Some(node_index) = self.app_config.profile_nodes.remove(name) {
            self.app_config.graph.remove_node(node_index);
        }

        Ok(())
    }

    /// Renames a profile and updates the in-memory state.
    pub fn rename_profile(&mut self, old_name: &str, new_name: &str) -> Result<(), Box<dyn Error>> {
        let old_path = self
            .base_path
            .join("profiles")
            .join(format!("{old_name}.toml"));
        let new_path = self
            .base_path
            .join("profiles")
            .join(format!("{new_name}.toml"));

        if !old_path.exists() {
            return Err(format!("Profile '{old_name}' not found.").into());
        }
        if new_path.exists() {
            return Err(format!("Profile '{new_name}' already exists.").into());
        }

        fs::rename(old_path, new_path)?;

        // Update in-memory state
        if let Some(profile) = self.app_config.profiles.remove(old_name) {
            self.app_config
                .profiles
                .insert(new_name.to_string(), profile);
        }
        if let Some(node_index) = self.app_config.profile_nodes.remove(old_name) {
            self.app_config.graph[node_index] = new_name.to_string();
            self.app_config
                .profile_nodes
                .insert(new_name.to_string(), node_index);
        }

        Ok(())
    }
}
