use super::models::Profile;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug)]
pub enum LoadError {
    Io(io::Error),
    Parse(toml::de::Error),
    NotFound(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Io(err) => write!(f, "IO error: {}", err),
            LoadError::Parse(err) => write!(f, "Parse error: {}", err),
            LoadError::NotFound(name) => write!(f, "Profile '{}' not found", name),
        }
    }
}

impl Error for LoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            LoadError::Io(err) => Some(err),
            LoadError::Parse(err) => Some(err),
            LoadError::NotFound(_) => None,
        }
    }
}

impl From<io::Error> for LoadError {
    fn from(err: io::Error) -> Self {
        LoadError::Io(err)
    }
}

impl From<toml::de::Error> for LoadError {
    fn from(err: toml::de::Error) -> Self {
        LoadError::Parse(err)
    }
}

pub fn scan_profile_names(path: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let mut names = Vec::new();
    if !path.exists() {
        return Ok(names);
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && path.extension().and_then(|s| s.to_str()) == Some("toml")
            && let Some(profile_name) = path.file_stem().and_then(|s| s.to_str())
        {
            names.push(profile_name.to_string());
        }
    }
    Ok(names)
}

pub fn load_profile_from_file(base_path: &Path, name: &str) -> Result<Profile, LoadError> {
    let path = base_path.join("profiles").join(format!("{name}.toml"));
    if !path.exists() {
        return Err(LoadError::NotFound(name.to_string()));
    }
    let content = fs::read_to_string(&path)?;
    let profile: Profile = toml::from_str(&content)?;
    Ok(profile)
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
