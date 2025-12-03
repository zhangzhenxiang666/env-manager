use crate::{config::ConfigManager, utils};

pub fn handle() -> Result<(), Box<dyn std::error::Error>> {
    let config_manager = ConfigManager::new()?;

    let global_profile = config_manager.read_global()?;

    let vars = global_profile.collect_vars(&config_manager)?;
    let script = utils::env::generate_export_script(&vars);

    println!("{script}");

    Ok(())
}
