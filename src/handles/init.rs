use crate::{
    config::{ConfigManager, models::Profile},
    core,
};
use std::collections::HashMap;

pub fn handle() -> Result<(), Box<dyn std::error::Error>> {
    let config_manager = ConfigManager::new()?;

    let global_profile = config_manager.read_global()?;

    let vars = collect_vars_for_init(global_profile, config_manager)?;
    let script = core::script::generate_export_script(&vars);

    println!("{script}");

    Ok(())
}

fn collect_vars_for_init(
    global_profile: Profile,
    config_manager: ConfigManager,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut vars = HashMap::new();
    for profile_name in &global_profile.profiles {
        if let Some(profile) = config_manager.read_profile(profile_name) {
            vars.extend(profile.variables.clone());
        } else {
            return Err(format!("Profile `{profile_name}` not found").into());
        }
    }
    vars.extend(global_profile.variables);
    Ok(vars)
}
