use crate::cli::ProfileCommands::{self, Add, Create, Delete, List, Remove, Rename};
use crate::config::ConfigManager;
use crate::config::models::Profile;
use crate::{cli::ProfileRenameArgs, utils::display};

pub fn handle(profile_commands: ProfileCommands) -> Result<(), Box<dyn std::error::Error>> {
    let mut config_manager = ConfigManager::new()?;
    match profile_commands {
        List { expand } => list(expand, &mut config_manager),
        Create { name } => create(name, &mut config_manager),
        Rename(args) => rename(args, &mut config_manager),
        Delete { name } => delete(name, &mut config_manager),
        Add { name, items } => add(name, items, &mut config_manager),
        Remove { name, items } => remove(name, items, &mut config_manager),
    }
}

fn list(
    expand: bool,
    config_manager: &mut ConfigManager,
) -> Result<(), Box<dyn std::error::Error>> {
    config_manager.load_all_profiles()?;
    let profile_names = config_manager.scan_profile_names()?;
    if profile_names.is_empty() {
        display::show_info("No profiles found.");
        return Ok(());
    }

    if expand {
        profile_names.display_expand(config_manager)?;
    } else {
        profile_names.display_simple(config_manager)?;
    }

    Ok(())
}

fn create(
    name: String,
    config_manager: &mut ConfigManager,
) -> Result<(), Box<dyn std::error::Error>> {
    if config_manager.profile_exists(&name) {
        return Err(format!("Profile `{name}` already exists").into());
    }
    if !validate_non_empty(&name) {
        return Err("Profile name cannot be empty".into());
    }

    if !validate_starts_with_non_digit(&name) {
        return Err("Profile name must start with a non-digit character".into());
    }
    let profile = Profile::new();
    config_manager.write_profile(&name, &profile)?;
    display::show_success(&format!("Profile '{name}' created successfully."));
    Ok(())
}

fn rename(
    rename_args: ProfileRenameArgs,
    config_manager: &mut ConfigManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let ProfileRenameArgs {
        src_name,
        dest_name,
    } = rename_args;

    if !validate_non_empty(&dest_name) {
        return Err("Profile name cannot be empty".into());
    }

    if !validate_starts_with_non_digit(&dest_name) {
        return Err("Profile name must start with a non-digit character".into());
    }

    // Since other profiles may depend on the profile being renamed,
    // all profiles need to be loaded to update their dependency references
    config_manager.load_all_profiles()?;

    config_manager.rename_profile_file(&src_name, &dest_name)?;

    // Find reverse dependencies and update them (Only checks loaded profiles)
    if let Some(dependents) = config_manager.get_parents(&src_name) {
        for dep in dependents {
            config_manager.update_profile_dependencies(&dep, &src_name, &dest_name);
            if let Some(profile) = config_manager.get_profile(&dep) {
                config_manager.write_profile(&dep, profile)?;
            }
        }
    }

    display::show_success(&format!(
        "Profile '{src_name}' renamed to '{dest_name}' successfully."
    ));
    Ok(())
}

fn delete(
    name: String,
    config_manager: &mut ConfigManager,
) -> Result<(), Box<dyn std::error::Error>> {
    // No dependency check as requested
    config_manager.delete_profile_file(&name)?;
    display::show_success(&format!("Profile '{name}' deleted successfully."));
    Ok(())
}

fn add(
    name: String,
    items: Vec<String>,
    config_manager: &mut ConfigManager,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load profile to ensure it exists and graph is populated
    config_manager
        .load_profile(&name)
        .map_err(|_| format!("Profile `{name}` does not exist"))?;

    for item in items {
        if let Some((key, value)) = item.split_once('=') {
            if let Some(profile) = config_manager.get_profile_mut(&name) {
                profile.add_variable(key, value);
            }
            display::show_success(&format!("Variable '{key}' added to profile '{name}'."));
        } else {
            let dependency_to_add = &item;

            // Load dependency to check existence
            if config_manager.load_profile(dependency_to_add).is_err() {
                return Err(format!(
                    "Profile `{dependency_to_add}` does not exist and cannot be added as a nested profile."
                )
                .into());
            }

            if name == *dependency_to_add {
                return Err("A profile cannot depend on itself.".into());
            }

            // Use the new `find_path` method for a more detailed error message.
            if let Some(mut path) = config_manager.find_path(dependency_to_add, &name) {
                path.push(dependency_to_add.to_string()); // Complete the cycle path for display
                return Err(format!(
                    "Adding '{dependency_to_add}' to '{name}' would create a circular dependency: {}",
                    path.join(" -> ")
                )
                .into());
            }

            if let Some(profile) = config_manager.get_profile_mut(&name) {
                profile.add_profile(dependency_to_add);
            }
            display::show_success(&format!(
                "Nested profile '{dependency_to_add}' added to profile '{name}'."
            ));
        }
    }

    if let Some(profile) = config_manager.get_profile(&name) {
        config_manager.write_profile(&name, profile)?;
    }

    Ok(())
}

fn remove(
    name: String,
    items: Vec<String>,
    config_manager: &mut ConfigManager,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load profile
    config_manager
        .load_profile(&name)
        .map_err(|_| format!("Profile `{name}` does not exist"))?;

    for item in items {
        let was_variable = if let Some(profile) = config_manager.get_profile_mut(&name) {
            profile.remove_variable(&item).is_some()
        } else {
            false
        };

        let was_profile = if let Some(profile) = config_manager.get_profile_mut(&name) {
            let original_len = profile.profiles.len();
            profile.remove_profile(&item);
            profile.profiles.len() < original_len
        } else {
            false
        };

        if was_variable {
            display::show_success(&format!("Variable '{item}' removed from profile '{name}'."));
        } else if was_profile {
            display::show_success(&format!(
                "Nested profile '{item}' removed from profile '{name}'."
            ));
        } else {
            display::show_info(&format!("Item '{item}' not found in profile '{name}'."));
        }
    }

    if let Some(profile) = config_manager.get_profile(&name) {
        config_manager.write_profile(&name, profile)?;
    }
    Ok(())
}

fn validate_non_empty(text: &str) -> bool {
    text.trim().is_empty()
}

fn validate_starts_with_non_digit(text: &str) -> bool {
    if let Some(first_char) = text.chars().next()
        && first_char.is_ascii_digit()
    {
        false
    } else {
        true
    }
}
