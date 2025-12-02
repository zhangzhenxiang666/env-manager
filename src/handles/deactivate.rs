use crate::config::ConfigManager;
use crate::core;
use std::collections::{HashMap, HashSet};

pub fn handle(items: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let config_manager = ConfigManager::new()?;

    // 1. Separate direct key-value pairs from profile names
    let (key_value_items, profile_items): (Vec<_>, Vec<_>) =
        items.into_iter().partition(|item| item.contains('='));

    // 2. Resolve dependencies for each specified profile to get a full list of variables to unset
    let mut all_profiles_to_load = Vec::new();
    let mut seen_profiles = HashSet::new();

    for profile_name in &profile_items {
        let ordered_deps = config_manager.resolve_dependencies(profile_name)?;
        for dep in ordered_deps {
            if seen_profiles.insert(dep.clone()) {
                all_profiles_to_load.push(dep);
            }
        }
    }
    // also add the initial profiles themselves
    for profile_name in &profile_items {
        if seen_profiles.insert(profile_name.clone()) {
            all_profiles_to_load.push(profile_name.clone());
        }
    }

    // 3. Collect variables from all resolved profiles
    let mut vars = HashMap::new();
    for profile_name in all_profiles_to_load {
        if let Some(profile) = config_manager.read_profile(&profile_name) {
            vars.extend(profile.variables.clone());
        } else {
            return Err(format!("Profile `{profile_name}` not found during deactivation").into());
        }
    }

    // 4. Add direct key-value pairs
    for item in key_value_items {
        if let Some((key, _)) = item.split_once('=') {
            if !key.is_empty() {
                vars.insert(key.to_string(), String::new()); // Value doesn't matter for unset
            }
        }
    }

    // 5. Generate and print the unset script
    let script = core::script::generate_unset_script(&vars);
    println!("{script}");

    Ok(())
}
