use crate::{config::ConfigManager, utils};
pub fn handle() -> Result<(), Box<dyn std::error::Error>> {
    let mut config_manager = ConfigManager::new()?;

    let global_profile = config_manager.read_global()?;

    for profile in global_profile.profiles.iter() {
        config_manager.load_profile(profile)?;
    }

    let vars = global_profile.collect_vars(&config_manager)?;
    let mut generate = utils::shell_generate::ShellGenerate::new();
    generate.export_from_map(&vars);
    generate.output();

    Ok(())
}
