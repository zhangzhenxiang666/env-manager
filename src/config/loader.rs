use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use crate::config::models::Profile;

pub fn load_profiles_from_dir(path: &Path) -> Result<HashMap<String, Profile>, Box<dyn Error>> {
    let mut profiles = HashMap::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
            if let Some(profile_name) = path.file_stem().and_then(|s| s.to_str()) {
                let content = fs::read_to_string(&path)?;
                let profile: Profile = toml::from_str(&content)?;
                profiles.insert(profile_name.to_string(), profile);
            }
        }
    }
    Ok(profiles)
}

pub fn read_global_config(base_path: &Path) -> Result<Profile, Box<dyn Error>> {
    let path = base_path.join("global.toml");
    if !path.exists() {
        return Ok(Profile::new());
    }

    let content = fs::read_to_string(path)?;
    if content.trim().is_empty() {
        return Ok(Profile::new());
    }

    Ok(toml::from_str(&content)?)
}

pub fn write_global_config(base_path: &Path, global: &Profile) -> Result<(), Box<dyn Error>> {
    let path = base_path.join("global.toml");
    let content = toml::to_string_pretty(global)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn read_profile<'a>(profiles: &'a HashMap<String, Profile>, name: &str) -> Option<&'a Profile> {
    profiles.get(name)
}

pub fn read_profile_mut<'a>(
    profiles: &'a mut HashMap<String, Profile>,
    name: &str,
) -> Option<&'a mut Profile> {
    profiles.get_mut(name)
}

pub fn write_profile(
    base_path: &Path,
    name: &str,
    profile: &Profile,
) -> Result<(), Box<dyn Error>> {
    let path = base_path.join("profiles").join(format!("{name}.toml"));
    let content = toml::to_string_pretty(profile)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn delete_profile_file(base_path: &Path, name: &str) -> Result<(), Box<dyn Error>> {
    let path = base_path.join("profiles").join(format!("{name}.toml"));
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn rename_profile_file(
    base_path: &Path,
    old_name: &str,
    new_name: &str,
) -> Result<(), Box<dyn Error>> {
    let old_path = base_path.join("profiles").join(format!("{old_name}.toml"));
    let new_path = base_path.join("profiles").join(format!("{new_name}.toml"));

    if !old_path.exists() {
        return Err(format!("Profile '{old_name}' not found.").into());
    }
    if new_path.exists() {
        return Err(format!("Profile '{new_name}' already exists.").into());
    }

    fs::rename(old_path, new_path)?;
    Ok(())
}
