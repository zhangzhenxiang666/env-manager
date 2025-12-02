use crate::{
    cli::{
        ProfileCommands::{self, Add, Create, Delete, List, Remove, Rename},
        ProfileRenameArgs,
    },
    config::{ConfigManager, models::Profile},
    core::display,
};
use daggy::Walker;

pub fn handle(profile_commands: ProfileCommands) -> Result<(), Box<dyn std::error::Error>> {
    let mut config_manager = ConfigManager::new()?;
    match profile_commands {
        List { expand } => list(expand, &config_manager),
        Create { name } => create(name, &mut config_manager),
        Rename(args) => rename(args, &mut config_manager),
        Delete { name } => delete(name, &mut config_manager),
        Add { name, items } => add(name, items, &mut config_manager),
        Remove { name, items } => remove(name, items, &mut config_manager),
    }
}

fn list(expand: bool, config_manager: &ConfigManager) -> Result<(), Box<dyn std::error::Error>> {
    let profile_names = config_manager.list_profile_names();
    if profile_names.is_empty() {
        display::show_info("No profiles found.");
        return Ok(());
    }

    if expand {
        profile_names.display_expand(config_manager)?;
    } else {
        profile_names.display_simple();
    }

    Ok(())
}

fn create(
    name: String,
    config_manager: &mut ConfigManager,
) -> Result<(), Box<dyn std::error::Error>> {
    if config_manager.has_profile(&name) {
        return Err(format!("Profile `{name}` already exists").into());
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

    config_manager.rename_profile(&src_name, &dest_name)?;
    display::show_success(&format!(
        "Profile '{src_name}' renamed to '{dest_name}' successfully."
    ));
    Ok(())
}

fn delete(
    name: String,
    config_manager: &mut ConfigManager,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(&node_index) = config_manager.app_config.graph.profile_nodes.get(&name) {
        let parents = config_manager.app_config.graph.graph.parents(node_index);
        let dependents: Vec<String> = parents
            .iter(&config_manager.app_config.graph.graph)
            .map(|(_, parent_index)| config_manager.app_config.graph.graph[parent_index].clone())
            .collect();

        if !dependents.is_empty() {
            return Err(format!(
                "Cannot delete profile '{name}' because it is used by: {}",
                dependents.join(", ")
            )
            .into());
        }
    }

    config_manager.delete_profile(&name)?;
    display::show_success(&format!("Profile '{name}' deleted successfully."));
    Ok(())
}

fn add(
    name: String,
    items: Vec<String>,
    config_manager: &mut ConfigManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut profile = config_manager
        .read_profile(&name)
        .ok_or_else(|| format!("Profile `{name}` does not exist"))?
        .clone();

    for item in items {
        if let Some((key, value)) = item.split_once('=') {
            profile.add_variable(key, value);
            display::show_success(&format!("Variable '{key}' added to profile '{name}'."));
        } else {
            let dependency_to_add = &item;

            if !config_manager.has_profile(dependency_to_add) {
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

            profile.add_profile(dependency_to_add);
            display::show_success(&format!(
                "Nested profile '{dependency_to_add}' added to profile '{name}'."
            ));
        }
    }

    config_manager.write_profile(&name, &profile)?;

    Ok(())
}

fn remove(
    name: String,
    items: Vec<String>,
    config_manager: &mut ConfigManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut profile = config_manager
        .read_profile(&name)
        .ok_or_else(|| format!("Profile `{name}` does not exist"))?
        .clone();

    for item in items {
        let was_variable = profile.remove_variable(&item).is_some();
        let original_len = profile.profiles.len();
        profile.remove_profile(&item);
        let was_profile = profile.profiles.len() < original_len;

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

    config_manager.write_profile(&name, &profile)?;
    Ok(())
}
