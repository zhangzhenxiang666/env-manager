use crate::config::{
    ConfigManager,
    models::{Profile, ProfileNames},
};
use colored::*;

impl ProfileNames {
    pub fn display_simple(&self) {
        if self.is_empty() {
            return;
        }

        eprintln!("{}", "Profiles:".yellow());
        let mut names_iter = self.iter().peekable();
        while let Some(name) = names_iter.next() {
            let is_last_profile = names_iter.peek().is_none();
            let branch_prefix = if is_last_profile {
                "└──"
            } else {
                "├──"
            };
            eprintln!("{} {}", branch_prefix, name.cyan());
        }
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
            eprintln!("{} {}", top_level_branch, name.cyan());

            if let Some(profile_cfg) = config_manager.read_profile(name) {
                let has_variables = !profile_cfg.variables.is_empty();
                let has_nested_profiles = !profile_cfg.profiles.is_empty();

                if !has_nested_profiles && !has_variables {
                    continue;
                }

                let current_level_indent = if is_last_top_level_profile {
                    "    "
                } else {
                    "│   "
                };

                // Display nested profiles section
                if has_nested_profiles {
                    let profiles_section_branch = if has_variables {
                        "├──" // If there are variables after, this is a middle branch
                    } else {
                        "└──" // If no variables after, this is the last branch
                    };
                    let colored_profiles: Vec<String> = profile_cfg
                        .profiles
                        .iter()
                        .map(|p| p.blue().to_string())
                        .collect();
                    eprintln!(
                        "{} {} {}: [{}]",
                        current_level_indent,
                        profiles_section_branch,
                        "profiles".yellow(),
                        colored_profiles.join(", ")
                    );
                }

                // Display variables section
                if has_variables {
                    let variables_section_branch = "└──"; // Variables will always be the last section for a profile in this display context
                    eprintln!(
                        "{} {} {}",
                        current_level_indent,
                        variables_section_branch,
                        "variables".yellow()
                    );

                    let mut vars_iter = profile_cfg.variables.iter().peekable();
                    while let Some((key, value)) = vars_iter.next() {
                        let is_last_var = vars_iter.peek().is_none();
                        let var_branch = if is_last_var {
                            "└──"
                        } else {
                            "├──"
                        };
                        eprintln!(
                            "{}    {} {} = {}",
                            current_level_indent,
                            var_branch,
                            key.green(),
                            format!("\"{value}\"").truecolor(180, 180, 180)
                        );
                    }
                }
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
            eprintln!("{} {}", profiles_prefix, "profiles".yellow());

            let mut profiles_iter = self.profiles.iter().peekable();
            let parent_pipe_prefix = if has_variables { "│   " } else { "    " };

            while let Some(profile_name) = profiles_iter.next() {
                let is_last_profile = profiles_iter.peek().is_none();
                let branch_prefix = if is_last_profile {
                    "└──"
                } else {
                    "├──"
                };
                eprintln!(
                    "{}{}{}",
                    parent_pipe_prefix,
                    branch_prefix,
                    profile_name.cyan()
                );

                if let Some(nested_profile) = config_manager.read_profile(profile_name) {
                    // Start of display_simple logic, adapted for indentation
                    let nested_has_profiles = !nested_profile.profiles.is_empty();
                    let nested_has_variables = !nested_profile.variables.is_empty();

                    if !nested_has_profiles && !nested_has_variables {
                        continue;
                    }

                    let nested_pipe_prefix = if is_last_profile { "    " } else { "│   " };
                    let current_level_indent = format!("{parent_pipe_prefix}{nested_pipe_prefix}");

                    if nested_has_profiles {
                        let nested_profiles_prefix = if nested_has_variables {
                            "├──"
                        } else {
                            "└──"
                        };

                        let colored_profiles: Vec<String> = nested_profile
                            .profiles
                            .iter()
                            .map(|p| p.blue().to_string())
                            .collect();

                        eprintln!(
                            "{}{} {}: [{}]",
                            current_level_indent,
                            nested_profiles_prefix,
                            "profiles".yellow(),
                            colored_profiles.join(", ")
                        );
                    }

                    if nested_has_variables {
                        let nested_vars_prefix = "└──";
                        eprintln!(
                            "{}{} {}",
                            current_level_indent,
                            nested_vars_prefix,
                            "variables".yellow()
                        );

                        let mut nested_var_iter = nested_profile.variables.iter().peekable();
                        while let Some((key, value)) = nested_var_iter.next() {
                            let var_prefix = if nested_var_iter.peek().is_some() {
                                "    ├──"
                            } else {
                                "    └──"
                            };
                            eprintln!(
                                "{}{} {}: {}",
                                current_level_indent,
                                var_prefix,
                                key.green(),
                                format!("\"{value}\"").truecolor(180, 180, 180)
                            );
                        }
                    }
                }
            }
        }

        if has_variables {
            eprintln!("└── {}", "variables".yellow());
            let mut vars_iter = self.variables.iter().peekable();
            while let Some((key, value)) = vars_iter.next() {
                let is_last_var = vars_iter.peek().is_none();
                let var_branch = if is_last_var {
                    "└──"
                } else {
                    "├──"
                };
                // Corrected indentation for variables: 4 spaces to align with "variables" label.
                eprintln!(
                    "    {var_branch} {} = {}",
                    key.green(),
                    format!("\"{value}\"").truecolor(180, 180, 180)
                );
            }
        }
        Ok(())
    }

    pub fn display_simple(&self) {
        let has_profiles = !self.profiles.is_empty();
        let has_variables = !self.variables.is_empty();

        if !has_profiles && !has_variables {
            return;
        }

        if has_profiles {
            let prefix = if has_variables {
                "├──"
            } else {
                "└──"
            };
            let colored_profiles: Vec<String> =
                self.profiles.iter().map(|p| p.blue().to_string()).collect();
            eprintln!(
                "{prefix} {}: [{}]",
                "profiles".yellow(),
                colored_profiles.join(", ")
            );
        }

        if has_variables {
            eprintln!("└── {}", "variables".yellow());
            let mut var_iter = self.variables.iter().peekable();
            while let Some((key, value)) = var_iter.next() {
                let prefix = if var_iter.peek().is_some() {
                    "    ├──"
                } else {
                    "    └──"
                };
                eprintln!(
                    "{} {}: {}",
                    prefix,
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
