use crate::{
    cli::GlobalCommands::{self, Add, List, Remove},
    config::ConfigManager,
};

pub fn handle(global_commands: GlobalCommands) -> Result<(), Box<dyn std::error::Error>> {
    let config_manager = ConfigManager::new()?;
    match global_commands {
        List { expand } => list(expand, config_manager),
        Add { items } => add(items, config_manager),
        Remove { items } => remove(items, config_manager),
    }
}

/// Handles the logic for listing the global configuration.
fn list(expand: bool, config_manager: ConfigManager) -> Result<(), Box<dyn std::error::Error>> {
    let global = config_manager.read_global()?;
    if expand {
        eprintln!("Global Config (tree view):");
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
    config_manager: ConfigManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut global = config_manager.read_global()?;
    for item in items {
        if let Some((key, value)) = item.split_once('=') {
            if !key.is_empty() {
                global.add_variable(key, value);
            }
        } else {
            if !config_manager.has_profile(&item) {
                return Err(format!("`{item}` Profile not found").into());
            }
            global.add_profile(&item);
        }
    }
    config_manager.write_global(&global)?;
    println!("Global configuration updated successfully.");
    Ok(())
}

/// Handles the logic for removing items from the global configuration.
fn remove(
    items: Vec<String>,
    config_manager: ConfigManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut global = config_manager.read_global()?;
    for item in items {
        global.remove_profile(&item);
        global.remove_variable(&item);
    }
    config_manager.write_global(&global)?;
    println!("Global configuration updated successfully.");
    Ok(())
}
