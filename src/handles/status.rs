use crate::cli::CommandsStatusArgs;
use crate::config::ConfigManager;
use crate::config::models::Profile;
use colored::*;
use std::collections::BTreeMap;
use std::error::Error;

#[derive(Debug)]
struct VarStatus {
    profile_value: String,
    shell_value: Option<String>,
}

#[derive(Debug, PartialEq)]
enum VarState {
    Match,
    Mismatch,
    ProfileOnly,
}

impl VarStatus {
    fn new(key: &str, profile_value: &str) -> Self {
        Self {
            profile_value: profile_value.to_string(),
            shell_value: std::env::var(key).ok(),
        }
    }

    fn state(&self) -> VarState {
        match &self.shell_value {
            Some(shell_val) => {
                if shell_val == &self.profile_value {
                    VarState::Match
                } else {
                    VarState::Mismatch
                }
            }
            None => VarState::ProfileOnly,
        }
    }
}

pub fn handle(args: CommandsStatusArgs) -> Result<(), Box<dyn Error>> {
    let config_manager = ConfigManager::new()?;
    for (i, profile_name) in args.profiles.iter().enumerate() {
        if !config_manager.has_profile(profile_name) {
            eprintln!(
                "{}",
                format!("Warning: Profile '{profile_name}' not found.").yellow()
            );
            continue;
        }

        let profile = config_manager.read_profile(profile_name).unwrap();
        let is_last_profile = i == args.profiles.len() - 1;
        let profile_prefix = if is_last_profile {
            "└──"
        } else {
            "├──"
        };

        eprintln!("{profile_prefix} {}", profile_name.cyan());

        let indent = if is_last_profile { "    " } else { "│   " };
        display_profile_status(profile, &config_manager, args.expand, indent)?;
    }

    Ok(())
}

fn display_profile_status(
    profile: &Profile,
    config_manager: &ConfigManager,
    expand: bool,
    indent: &str,
) -> Result<(), Box<dyn Error>> {
    let mut statuses = BTreeMap::new();
    for (key, value) in &profile.variables {
        statuses.insert(key.clone(), VarStatus::new(key, value));
    }

    let has_nested_profiles = expand && !profile.profiles.is_empty();
    let mut max_key_len = statuses.keys().map(|k| k.len()).max().unwrap_or(0);
    if has_nested_profiles {
        max_key_len = max_key_len.max("profiles".len());
    }

    let mut status_iter = statuses.iter().peekable();
    while let Some((key, status)) = status_iter.next() {
        let is_last = status_iter.peek().is_none() && !has_nested_profiles;
        let prefix = if is_last { "└──" } else { "├──" };
        let line = format!("{indent}{prefix}");
        let key_part = format!("{key}:");
        let padded_key_part = format!("{:<width$}", key_part, width = max_key_len + 2);

        match status.state() {
            VarState::Match => {
                eprintln!(
                    "{} {}{}",
                    line,
                    padded_key_part.green(),
                    status.profile_value
                );
            }
            VarState::Mismatch => {
                let shell_val = status.shell_value.as_ref().unwrap();
                let output = format!(
                    "{} -> {}",
                    status.profile_value.strikethrough(),
                    shell_val.yellow()
                );
                eprintln!("{} {}{}", line, padded_key_part.yellow(), output);
            }
            VarState::ProfileOnly => {
                let output = format!("{} {}", status.profile_value, "[Unset in shell]".blue());
                eprintln!("{} {}{}", line, padded_key_part.blue(), output);
            }
        }
    }

    if has_nested_profiles {
        let prefix = "└──";
        let profiles_key = format!("{:<width$}", "profiles:", width = max_key_len + 2);
        let line = format!("{indent}{prefix}");
        eprintln!("{} {}", line, profiles_key.magenta());

        let nested_indent = format!("{indent}    ");

        let mut profile_iter = profile.profiles.iter().peekable();
        while let Some(nested_name) = profile_iter.next() {
            if let Some(nested_profile) = config_manager.read_profile(nested_name) {
                let is_last_nested = profile_iter.peek().is_none();
                let nested_profile_prefix = if is_last_nested {
                    "└──"
                } else {
                    "├──"
                };
                eprintln!(
                    "{}{nested_profile_prefix} {}",
                    nested_indent,
                    nested_name.cyan()
                );

                let last_nested_indent = if is_last_nested { "    " } else { "│   " };
                let final_indent = format!("{nested_indent}{last_nested_indent}");
                display_profile_status(nested_profile, config_manager, false, &final_indent)?;
            }
        }
    }

    Ok(())
}
