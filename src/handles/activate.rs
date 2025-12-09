use crate::config::ConfigManager;
use crate::utils;
use crate::utils::display;
use std::collections::HashMap;

pub fn handle(items: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut config_manager = ConfigManager::new()?;

    // Separate direct key-value pairs from profile names
    let (key_value_items, profile_items): (Vec<_>, Vec<_>) =
        items.into_iter().partition(|item| item.contains('='));

    let mut vars = HashMap::new();

    for profile_name in &profile_items {
        config_manager.load_profile(profile_name)?;
        vars.extend(
            config_manager
                .get_profile(profile_name)
                .unwrap()
                .collect_vars(&config_manager)?,
        );
    }

    // Add direct key-value pairs, potentially overwriting profile variables
    let mut direct_keys = Vec::new();
    for item in key_value_items {
        if let Some((key, value)) = item.split_once('=')
            && !key.is_empty()
        {
            vars.insert(key.to_string(), value.to_string());
            direct_keys.push(key.to_string());
        }
    }

    // 5. Generate and print the script
    let script = utils::env::generate_export_script(&vars);
    println!("{script}");

    if !profile_items.is_empty() {
        display::show_success(&format!(
            "Successfully activated profiles: {}",
            profile_items.join(", ")
        ));
    }

    if !direct_keys.is_empty() {
        display::show_success(&format!(
            "Set environment variables: {}",
            direct_keys.join(", ")
        ));
    }

    Ok(())
}
