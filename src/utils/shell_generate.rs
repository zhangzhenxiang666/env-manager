use crate::SHELL_MARK;
use std::{collections::HashMap, env};

#[derive(Debug, Clone, Copy)]
enum ShellType {
    Posix,
    Fish,
    #[cfg(windows)]
    PowerShell,
    #[cfg(windows)]
    Cmd,
}

impl ShellType {
    fn detect() -> Self {
        if let Ok(shell_type) = env::var("EM_SHELL") {
            return match shell_type.to_lowercase().as_str() {
                "fish" => ShellType::Fish,
                #[cfg(windows)]
                "pwsh" | "powershell" => ShellType::PowerShell,
                #[cfg(windows)]
                "cmd" => ShellType::Cmd,
                "bash" | "zsh" | "sh" => ShellType::Posix,
                _ => ShellType::Posix,
            };
        }

        ShellType::Posix
    }

    fn export_cmd(&self, key: &str, value: &str) -> String {
        match self {
            Self::Posix => {
                let escaped_value = value.replace('\'', r"'\''");
                format!("export {key}='{escaped_value}'")
            }
            Self::Fish => {
                let escaped_value = value.replace('\\', r"\\").replace('\'', r"\'");
                format!("set -x {key} '{escaped_value}'")
            }
            #[cfg(windows)]
            Self::PowerShell => {
                let escaped_value = value
                    .replace('`', "``")
                    .replace('"', "`\"")
                    .replace('$', "`$");
                format!("$env:{key}=\"{escaped_value}\"")
            }
            #[cfg(windows)]
            Self::Cmd => {
                let escaped_value = value
                    .replace('%', "%%")
                    .replace('^', "^^")
                    .replace('&', "^&")
                    .replace('|', "^|")
                    .replace('<', "^<")
                    .replace('>', "^>")
                    .replace('"', "^\"");
                format!("set {key}={escaped_value}")
            }
        }
    }

    fn unset_cmd(&self, key: &str) -> String {
        match self {
            Self::Posix => format!("unset {key}"),
            Self::Fish => format!("set -e {key}"),
            #[cfg(windows)]
            Self::PowerShell => format!("Remove-Item Env:{key}"),
            #[cfg(windows)]
            Self::Cmd => format!("set {key}="),
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
