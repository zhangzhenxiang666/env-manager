use crate::SHELL_MARK;
use crate::utils::shell_generate::ShellType;

const POSIX_SHELL_WRAPPER_TEMPLATE: &str = include_str!("../../templates/posix.sh");
const FISH_SHELL_WRAPPER_TEMPLATE: &str = include_str!("../../templates/fish.fish");
const POWERSHELL_WRAPPER_TEMPLATE: &str = include_str!("../../templates/powershell.ps1");

pub fn handle(shell: String, print_full_init: bool) -> Result<(), Box<dyn std::error::Error>> {
    let shell_type = ShellType::try_from(shell.as_str())?;
    // Special handling for cmd. Usually we don't put .exe in env-manage binary path for other logic,
    let mut exe_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => std::path::PathBuf::from("~/.config/env-mange/bin/env-mange"),
    };
    // Ensure we use the .exe extension on Windows if not present (though current_exe usually has it)
    if cfg!(windows) && exe_path.extension().is_none() {
        exe_path.set_extension("exe");
    }

    match shell_type {
        ShellType::Bash => init_bash(exe_path, print_full_init),
        ShellType::Zsh => init_zsh(exe_path, print_full_init),
        ShellType::Fish => init_fish(exe_path, print_full_init),
        ShellType::PowerShell => init_powershell(exe_path, print_full_init),
    }
}

fn init_bash(
    exe_path: std::path::PathBuf,
    print_full_init: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !print_full_init {
        print!(
            "eval -- \"$({} init bash --print-full-init)\"",
            exe_path.display()
        );
    } else {
        print!(
            "{}",
            generate_posix_shell_wrapper("bash", exe_path.to_str().unwrap())
        );
    }
    Ok(())
}

fn init_zsh(
    exe_path: std::path::PathBuf,
    print_full_init: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !print_full_init {
        print!(
            "eval -- \"$({} init zsh --print-full-init)\"",
            exe_path.display()
        );
    } else {
        print!(
            "{}",
            generate_posix_shell_wrapper("zsh", exe_path.to_str().unwrap())
        );
    }
    Ok(())
}

fn init_fish(
    exe_path: std::path::PathBuf,
    print_full_init: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !print_full_init {
        print!(
            "{} init fish --print-full-init | source",
            exe_path.display()
        );
    } else {
        print!(
            "{}",
            generate_fish_shell_wrapper("fish", exe_path.to_str().unwrap())
        );
    }
    Ok(())
}

fn init_powershell(
    exe_path: std::path::PathBuf,
    print_full_init: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !print_full_init {
        print!(
            "Invoke-Expression (& '{}' init powershell --print-full-init | Out-String)",
            exe_path.display()
        );
    } else {
        print!(
            "{}",
            generate_powershell_wrapper("powershell", exe_path.to_str().unwrap())
        );
    }
    Ok(())
}

fn generate_posix_shell_wrapper(shell_type: &str, binary_path: &str) -> String {
    let marker_length = SHELL_MARK.len();

    POSIX_SHELL_WRAPPER_TEMPLATE
        .replace("{{SHELL_TYPE}}", shell_type)
        .replace("{{BINARY_PATH}}", binary_path)
        .replace("{{SHELL_CMD_MARKER}}", SHELL_MARK)
        .replace("{{MARKER_LENGTH}}", &marker_length.to_string())
}

fn generate_fish_shell_wrapper(shell_type: &str, binary_path: &str) -> String {
    let marker_length = SHELL_MARK.len();
    // fish string sub is 1-based index
    let marker_length_plus_one = marker_length + 1;

    FISH_SHELL_WRAPPER_TEMPLATE
        .replace("{{SHELL_TYPE}}", shell_type)
        .replace("{{BINARY_PATH}}", binary_path)
        .replace("{{SHELL_CMD_MARKER}}", SHELL_MARK)
        .replace(
            "{{MARKER_LENGTH_PLUS_ONE}}",
            &marker_length_plus_one.to_string(),
        )
}

fn generate_powershell_wrapper(shell_type: &str, binary_path: &str) -> String {
    let marker_length = SHELL_MARK.len();

    let binary_path = binary_path.replace('"', "`\"");

    POWERSHELL_WRAPPER_TEMPLATE
        .replace("{{SHELL_TYPE}}", shell_type)
        .replace("{{BINARY_PATH}}", &binary_path)
        .replace("{{SHELL_CMD_MARKER}}", SHELL_MARK)
        .replace("{{MARKER_LENGTH}}", &marker_length.to_string())
}
