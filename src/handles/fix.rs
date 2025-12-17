use crate::config::{ConfigManager, graph::DependencyError};
use crate::utils::display;

pub fn handle() -> Result<(), Box<dyn std::error::Error>> {
    let mut config_manager = ConfigManager::new()?;
    let profile_names = config_manager.scan_profile_names()?;

    let mut fixed_count = 0;

    for name in profile_names.iter() {
        // Attempt load
        match config_manager.load_profile(name) {
            Ok(_) => continue,
            Err(e) => {
                if fix_profile(&mut config_manager, &e)? {
                    fixed_count += 1;
                } else {
                    display::show_error(&format!("Could not fix issue in '{name}': {e}"));
                }
            }
        }
    }

    if fixed_count > 0 {
        display::show_success(&format!("Fixed {fixed_count} profiles."));
    } else {
        display::show_info("No fixable issues found.");
    }

    Ok(())
}

fn fix_profile(
    config_manager: &mut ConfigManager,
    error: &DependencyError,
) -> Result<bool, Box<dyn std::error::Error>> {
    match error {
        DependencyError::DependencyChain { profile, cause } => {
            // Use pattern matching to check if the cause is immediately a missing profile
            if let DependencyError::ProfileNotFound(target) = &**cause {
                // 'profile' references 'target' which is missing. Fix 'profile'.
                return remove_dependency_from_file(config_manager, profile, target);
            }
            // Otherwise recurse down the chain
            fix_profile(config_manager, cause)
        }
        DependencyError::DependencyNotFound(parent, dep_name) => {
            remove_dependency_from_file(config_manager, parent, dep_name)
        }
        DependencyError::CircularDependency(path) => {
            if path.len() < 2 {
                return Ok(false);
            }

            let target = path.last().unwrap();
            let source = path.get(path.len() - 2).unwrap();

            remove_dependency_from_file(config_manager, source, target)
        }
        DependencyError::MultipleErrors(errors) => {
            let mut fixed_any = false;
            for e in errors {
                if fix_profile(config_manager, e)? {
                    fixed_any = true;
                }
            }
            Ok(fixed_any)
        }
        DependencyError::ProfileNotFound(_) => {
            // Top level profile not found? Can't fix.
            Ok(false)
        }
        DependencyError::ProfileIoError(_, _) => {
            // IO error? Can't fix automatically.
            Ok(false)
        }
        DependencyError::ProfileParseError(_, _) => {
            // Parse error? Can't fix automatically.
            Ok(false)
        }
    }
}

fn remove_dependency_from_file(
    config_manager: &mut ConfigManager,
    profile_name: &str,
    dep_name: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    // We need to read the profile file raw because load_profile failed.
    // If the file itself is missing, we can't edit it.
    if !config_manager
        .base_path()
        .join("profiles")
        .join(format!("{profile_name}.toml"))
        .exists()
    {
        return Ok(false);
    }

    let mut profile =
        crate::config::loader::load_profile_from_file(config_manager.base_path(), profile_name)?;

    if profile.profiles.contains(dep_name) {
        profile.remove_profile(dep_name);
        config_manager.write_profile(profile_name, &profile)?;

        config_manager.remove_profile(profile_name);

        config_manager.remove_profile_node(profile_name)?;

        crate::utils::display::show_success(&format!(
            "Removed dependency '{dep_name}' from profile '{profile_name}'",
        ));
        return Ok(true);
    }
    Ok(false)
}
