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
    pub profiles: HashMap<String, Profile>,
    pub graph: ProfileGraph,
}

impl AppConfig {
    pub fn new(profiles: HashMap<String, Profile>, graph: ProfileGraph) -> Self {
        Self { profiles, graph }
    }
}

pub struct ConfigManager {
    pub app_config: AppConfig,
    pub base_path: PathBuf,
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

    pub fn resolve_dependencies(&self, profile_name: &str) -> Result<Vec<String>, DependencyError> {
        self.app_config.graph.resolve_dependencies(profile_name)
    }

    pub fn find_path(&self, start_node: &str, end_node: &str) -> Option<Vec<String>> {
        self.app_config.graph.find_path(start_node, end_node)
    }

    pub fn read_global(&self) -> Result<Profile, Box<dyn Error>> {
        loader::read_global_config(&self.base_path)
    }

    pub fn write_global(&self, global: &Profile) -> Result<(), Box<dyn Error>> {
        loader::write_global_config(&self.base_path, global)
    }

    pub fn read_profile(&self, name: &str) -> Option<&Profile> {
        loader::read_profile(&self.app_config.profiles, name)
    }

    pub fn read_profile_mut(&mut self, name: &str) -> Option<&mut Profile> {
        loader::read_profile_mut(&mut self.app_config.profiles, name)
    }

    pub fn write_profile(&self, name: &str, profile: &Profile) -> Result<(), Box<dyn Error>> {
        loader::write_profile(&self.base_path, name, profile)?;
        Ok(())
    }

    pub fn list_profile_names(&self) -> ProfileNames {
        let names = self.app_config.profiles.keys().cloned().collect();
        ProfileNames(names)
    }

    pub fn has_profile(&self, name: &str) -> bool {
        self.app_config.profiles.contains_key(name)
    }

    pub fn delete_profile(&self, name: &str) -> Result<(), Box<dyn Error>> {
        loader::delete_profile_file(&self.base_path, name)?;
        Ok(())
    }

    pub fn rename_profile(&self, old_name: &str, new_name: &str) -> Result<(), Box<dyn Error>> {
        loader::rename_profile_file(&self.base_path, old_name, new_name)?;
        Ok(())
    }
}
