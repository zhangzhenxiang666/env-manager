use crate::config::ConfigManager;
use crate::utils::display;

pub fn handle() -> Result<(), Box<dyn std::error::Error>> {
    let mut config_manager = ConfigManager::new()?;

    let profile_names = config_manager.scan_profile_names()?;

    let mut found_issues = false;

    for name in profile_names.iter() {
        if let Err(e) = crate::utils::validate_profile_name(name) {
            found_issues = true;
            display::show_warning(&format!("Invalid profile name '{name}': {e}"));
        }

        if let Err(e) = config_manager.load_profile(name) {
            found_issues = true;
            match e {
                crate::config::graph::DependencyError::MultipleErrors(errors) => {
                    for err in errors {
                        display::show_error(&format!("{err}"));
                    }
                }
                _ => {
                    display::show_error(&format!("{e}"));
                }
            }
        }
    }

    if !found_issues {
        display::show_success("All profiles are valid.");
    } else {
        // Return an error to indicate failure? Or just exit?
        // User said "report ... errors".
        return Err("Found issues in profiles.".into());
    }

    Ok(())
}
