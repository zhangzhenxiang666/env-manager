use crate::cli::GlobalCommands::{self, Add, Clean, List, Remove};
use crate::config::ConfigManager;
use crate::utils;
use crate::utils::display::{show_info, show_success};

pub fn handle(global_commands: GlobalCommands) -> Result<(), Box<dyn std::error::Error>> {
    let mut config_manager = ConfigManager::new()?;
    match global_commands {
        List { expand } => list(expand, &mut config_manager),
        Add { items } => add(items, &mut config_manager),
        Remove { items } => remove(items, &mut config_manager),
        Clean => clean(&mut config_manager),
    }
}

/// Handles the logic for listing the global configuration.
fn list(
    expand: bool,
    config_manager: &mut ConfigManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let global = config_manager.read_global()?;

    if global.is_empty() {
        show_info("Global configuration is empty.");
        return Ok(());
    }

    for profile in global.profiles.iter() {
        config_manager.load_profile(profile)?;
    }

    if expand {
        eprintln!("Global Config (expand view):");
        global.display_expand(&config_manager)?;
    } else {
        eprintln!("global");
        global.display_simple();
    }
    Ok(())
}

/// Handles the logic for adding items to the global configuration.
fn add(
    items: Vec<String>,
    config_manager: &mut ConfigManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut global = config_manager.read_global()?;
    let mut added_profiles = Vec::new();
    let mut added_variables = Vec::new();

    for item in items {
        if let Some((key, value)) = item.split_once('=') {
            if !key.is_empty() {
                global.add_variable(key, value);
                added_variables.push(key.to_string());
            }
        } else {
            config_manager.load_profile(&item)?;
            global.add_profile(&item);
            added_profiles.push(item);
        }
    }

    if !added_profiles.is_empty() || !added_variables.is_empty() {
        config_manager.write_global(&global)?;
    }

    if !added_profiles.is_empty() {
        show_success(&format!(
            "Added profiles to global config: {}",
            added_profiles.join(", ")
        ));
    }
    if !added_variables.is_empty() {
        show_success(&format!(
            "Added variables to global config: {}",
            added_variables.join(", ")
        ));
    }
    Ok(())
}

/// Handles the logic for removing items from the global configuration.
fn remove(
    items: Vec<String>,
    config_manager: &ConfigManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut global = config_manager.read_global()?;
    let mut removed_profiles = Vec::new();
    let mut removed_variables = Vec::new();
    let mut not_found_items = Vec::new();

    for item in items {
        if global.remove_variable(&item).is_some() {
            removed_variables.push(item);
        } else {
            let original_len = global.profiles.len();
            global.remove_profile(&item);
            if global.profiles.len() < original_len {
                removed_profiles.push(item);
            } else {
                not_found_items.push(item);
            }
        }
    }

    if !removed_profiles.is_empty() || !removed_variables.is_empty() {
        config_manager.write_global(&global)?;
    }

    if !removed_profiles.is_empty() {
        show_success(&format!(
            "Removed profiles from global config: {}",
            removed_profiles.join(", ")
        ));
    }
    if !removed_variables.is_empty() {
        show_success(&format!(
            "Removed variables from global config: {}",
            removed_variables.join(", ")
        ));
    }
    if !not_found_items.is_empty() {
        show_info(&format!(
            "Items not found in global config: {}",
            not_found_items.join(", ")
        ));
    }
    Ok(())
}

/// Handles the logic for cleaning the global configuration.
fn clean(config_manager: &mut ConfigManager) -> Result<(), Box<dyn std::error::Error>> {
    let mut global_profile = config_manager.read_global()?;
    for profile in global_profile.profiles.iter() {
        config_manager.load_profile(profile)?;
    }
    let vars = global_profile.collect_vars(&config_manager)?;
    let script = utils::env::generate_unset_script(&vars);

    global_profile.clear();
    config_manager.write_global(&global_profile)?;

    println!("{script}");

    show_success("Global configuration cleaned successfully.");
    Ok(())
}
