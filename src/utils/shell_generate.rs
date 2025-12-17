use crate::SHELL_MARK;
use std::{collections::HashMap, env};

#[derive(Debug, Clone, Copy)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

impl ShellType {
    fn unsupported_shell_error(shell: &str) -> String {
        const SUPPORTED: &[&str] = &["bash", "zsh", "fish", "powershell", "pwsh"];

        let shells_list = SUPPORTED
            .iter()
            .map(|s| format!("* {}", s))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "{shell} is not yet supported by env-manage.\n\
            For the time being, we support the following shells:\n\
            {shells_list}
            "
        )
    }
    fn detect() -> Self {
        if let Ok(shell_type) = env::var("EM_SHELL") {
            return match shell_type.to_lowercase().as_str() {
                "fish" => ShellType::Fish,
                "pwsh" | "powershell" => ShellType::PowerShell,
                "bash" => ShellType::Bash,
                "zsh" => ShellType::Zsh,
                _ => ShellType::Bash,
            };
        }
        ShellType::Bash
    }

    fn export_cmd(&self, key: &str, value: &str) -> String {
        match self {
            Self::Bash => {
                let escaped_value = value.replace('\'', r"'\''");
                format!("export {key}='{escaped_value}'")
            }
            Self::Zsh => {
                let escaped_value = value.replace('\'', r"'\''");
                format!("export {key}='{escaped_value}'")
            }
            Self::Fish => {
                let escaped_value = value.replace('\\', r"\\").replace('\'', r"\'");
                format!("set -gx {key} '{escaped_value}'")
            }
            Self::PowerShell => {
                let escaped_value = value
                    .replace('`', "``")
                    .replace('"', "`\"")
                    .replace('$', "`$");
                format!("$env:{key}=\"{escaped_value}\"")
            }
        }
    }

    fn unset_cmd(&self, key: &str) -> String {
        match self {
            Self::Bash | Self::Zsh => format!("unset {key}"),
            Self::Fish => format!("set -e {key}"),
            Self::PowerShell => format!("Remove-Item Env:{key}"),
        }
    }
}

impl TryFrom<&str> for ShellType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "fish" => Ok(ShellType::Fish),
            "pwsh" | "powershell" => Ok(ShellType::PowerShell),
            "bash" => Ok(ShellType::Bash),
            "zsh" => Ok(ShellType::Zsh),
            _ => Err(Self::unsupported_shell_error(value)),
        }
    }
}

pub struct ShellGenerate {
    shell: ShellType,
    commands: Vec<String>,
}

impl Default for ShellGenerate {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellGenerate {
    pub fn new() -> Self {
        ShellGenerate {
            shell: ShellType::detect(),
            commands: Vec::new(),
        }
    }

    pub fn export(&mut self, key: &str, value: &str) -> &mut Self {
        self.commands.push(self.shell.export_cmd(key, value));
        self
    }

    pub fn unset(&mut self, key: &str) -> &mut Self {
        self.commands.push(self.shell.unset_cmd(key));
        self
    }

    pub fn export_from_map(&mut self, vars: &HashMap<String, String>) -> &mut Self {
        for (key, value) in vars {
            self.export(key, value);
        }
        self
    }

    pub fn unset_from_map(&mut self, vars: &HashMap<String, String>) -> &mut Self {
        for key in vars.keys() {
            self.unset(key);
        }
        self
    }

    pub fn build(&self) -> String {
        if self.commands.is_empty() {
            return String::new();
        }

        format!("{SHELL_MARK}\n{}", self.commands.join("\n"))
    }

    pub fn output(&self) {
        let result = self.build();
        if !result.is_empty() {
            print!("{}", result);
        }
    }
}
