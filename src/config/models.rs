use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::config::ConfigManager;

// Represents a single profile with its environment variables.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Profile {
    // Using a HashMap to store the key-value pairs for the environment.
    #[serde(default)]
    pub variables: HashMap<String, String>,
    #[serde(default)]
    pub profiles: Vec<String>,
}

pub struct ProfileNames(pub Vec<String>);

impl Profile {
    pub fn new() -> Self {
        Profile::default()
    }

    pub fn clear(&mut self) {
        self.variables.clear();
        self.profiles.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.variables.is_empty() && self.profiles.is_empty()
    }

    pub fn add_profile(&mut self, name: &str) {
        self.profiles.push(name.to_string());
    }

    pub fn remove_profile(&mut self, name: &str) {
        self.profiles.retain(|p| p != name);
    }

    pub fn add_variable(&mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());
    }

    pub fn remove_variable(&mut self, key: &str) -> Option<String> {
        self.variables.remove(key)
    }

    pub fn collect_vars(
        &self,
        config_manager: &ConfigManager,
    ) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let mut all_profiles_to_load = Vec::new();
        let mut seen_profiles = HashSet::new();

        for profile_name in &self.profiles {
            let ordered_deps = config_manager.resolve_dependencies(profile_name)?;
            for dep in ordered_deps {
                if seen_profiles.insert(dep.clone()) {
                    all_profiles_to_load.push(dep);
                }
            }
        }

        // also add the initial profiles themselves
        for profile_name in &self.profiles {
            if seen_profiles.insert(profile_name.clone()) {
                all_profiles_to_load.push(profile_name.clone());
            }
        }

        //  Collect variables from all resolved profiles in order
        let mut vars = HashMap::new();
        for profile_name in all_profiles_to_load {
            if let Some(profile) = config_manager.read_profile(&profile_name) {
                vars.extend(profile.variables.clone());
            } else {
                // This should ideally not happen if resolve_dependencies works correctly
                return Err(format!("Profile `{profile_name}` not found during activation").into());
            }
        }

        vars.extend(self.variables.clone());

        Ok(vars)
    }
}

impl std::ops::Deref for ProfileNames {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
