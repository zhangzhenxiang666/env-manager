use crate::config::ConfigManager;
use crate::config::models::{Profile, ProfileNames};
use colored::*;

impl ProfileNames {
    pub fn display_simple(
        &self,
        config_manager: &ConfigManager,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_empty() {
            return Ok(());
        }

        eprintln!("{}", "Profiles:".yellow());
        let mut names_iter = self.iter().peekable();
        while let Some(name) = names_iter.next() {
            let is_last_top_level_profile = names_iter.peek().is_none();
            let top_level_branch = if is_last_top_level_profile {
                "└──"
            } else {
                "├──"
            };
            eprintln!("{top_level_branch} {}", name.cyan());

            if let Some(profile_cfg) = config_manager.read_profile(name) {
                let current_level_indent = if is_last_top_level_profile {
                    "    "
                } else {
                    "│   "
                };
                profile_cfg.display_simple_with_indent(current_level_indent);
            }
        }
        Ok(())
    }

    pub fn display_expand(
        &self,
        config_manager: &ConfigManager,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_empty() {
            return Ok(());
        }

        eprintln!("{}", "Profiles:".yellow());
        let mut names_iter = self.iter().peekable();
        while let Some(name) = names_iter.next() {
            let is_last_top_level_profile = names_iter.peek().is_none();
            let top_level_branch = if is_last_top_level_profile {
                "└──"
            } else {
                "├──"
            };
            eprintln!("{top_level_branch} {}", name.cyan());

            if let Some(profile_cfg) = config_manager.read_profile(name) {
                let current_level_indent = if is_last_top_level_profile {
                    "    "
                } else {
                    "│   "
                };
                profile_cfg.display_expand_with_indent(config_manager, current_level_indent)?;
            }
        }
        Ok(())
    }
}

impl Profile {
    pub fn display_expand(
        &self,
        config_manager: &ConfigManager,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.display_expand_with_indent(config_manager, "")
    }

    pub fn display_expand_with_indent(
        &self,

        config_manager: &ConfigManager,

        indent: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let has_profiles = !self.profiles.is_empty();

        let has_variables = !self.variables.is_empty();

        if !has_profiles && !has_variables {
            return Ok(());
        }

        if has_profiles {
            let profiles_prefix = if has_variables {
                "├──"
            } else {
                "└──"
            };

            eprintln!("{indent}{profiles_prefix}{}", "profiles".yellow());

            let mut profiles_iter = self.profiles.iter().peekable();

            let parent_pipe_prefix = if has_variables { "│   " } else { "    " };

            while let Some(profile_name) = profiles_iter.next() {
                let is_last_profile = profiles_iter.peek().is_none();

                let branch_prefix = if is_last_profile {
                    "└──"
                } else {
                    "├──"
                };

                let next_level_base_indent = format!("{indent}{parent_pipe_prefix}");

                eprintln!(
                    "{next_level_base_indent}{branch_prefix}{}",
                    profile_name.cyan()
                );

                if let Some(nested_profile) = config_manager.read_profile(profile_name) {
                    let nested_pipe_prefix = if is_last_profile { "    " } else { "│   " };

                    let nested_indent = format!("{next_level_base_indent}{nested_pipe_prefix}");

                    if nested_profile.profiles.is_empty() {
                        nested_profile
                            .display_expand_with_indent(config_manager, &nested_indent)?;
                    } else {
                        nested_profile.display_simple_with_indent(&nested_indent);
                    }
                }
            }
        }

        if has_variables {
            let variables_prefix = "└──";

            eprintln!("{}{} {}", indent, variables_prefix, "variables".yellow());

            let mut vars_iter = self.variables.iter().peekable();

            let var_indent = format!("{indent}    ");

            while let Some((key, value)) = vars_iter.next() {
                let is_last_var = vars_iter.peek().is_none();

                let var_branch = if is_last_var {
                    "└──"
                } else {
                    "├──"
                };

                eprintln!(
                    "{var_indent}{var_branch} {} = {}",
                    key.green(),
                    format!("\"{value}\"").truecolor(180, 180, 180)
                );
            }
        }

        Ok(())
    }

    pub fn display_simple(&self) {
        self.display_simple_with_indent("");
    }

    pub fn display_simple_with_indent(&self, indent: &str) {
        let has_profiles = !self.profiles.is_empty();
        let has_variables = !self.variables.is_empty();

        if !has_profiles && !has_variables {
            return;
        }

        if has_profiles {
            let profiles_prefix = if has_variables {
                "├──"
            } else {
                "└──"
            };
            let colored_profiles: Vec<String> =
                self.profiles.iter().map(|p| p.blue().to_string()).collect();
            eprintln!(
                "{indent}{profiles_prefix} {}: [{}]",
                "profiles".yellow(),
                colored_profiles.join(", ")
            );
        }

        if has_variables {
            let variables_prefix = "└──";
            eprintln!("{}{} {}", indent, variables_prefix, "variables".yellow());
            let mut var_iter = self.variables.iter().peekable();
            let var_indent = format!("{indent}    ");
            while let Some((key, value)) = var_iter.next() {
                let prefix = if var_iter.peek().is_some() {
                    "├──"
                } else {
                    "└──"
                };
                eprintln!(
                    "{var_indent}{prefix} {} = {}",
                    key.green(),
                    format!("\"{value}\"").truecolor(180, 180, 180)
                );
            }
        }
    }
}

pub fn show_success(message: &str) {
    eprintln!("{}", format!("✔ {message}").green());
}

pub fn show_error(message: &str) {
    eprintln!("{}", format!("✗ {message}").red());
}

pub fn show_info(message: &str) {
    eprintln!("{}", format!("ℹ {message}").blue());
}
