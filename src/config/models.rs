use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ops::Deref};

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
}

impl Deref for ProfileNames {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
