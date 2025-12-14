use crate::SHELL_MARK;
use crate::utils::shell_generate::ShellType;

pub fn handle(shell: String, print_full_init: bool) -> Result<(), Box<dyn std::error::Error>> {
    let shell_type = ShellType::try_from(shell.as_str())?;
    let exe_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => std::path::PathBuf::from("~/.config/env-mange/bin/env-mange"),
    };
    match shell_type {
        ShellType::Bash => init_bash(exe_path, print_full_init),
        ShellType::Zsh => init_zsh(exe_path, print_full_init),
        ShellType::Fish => init_fish(exe_path, print_full_init),
        #[cfg(target_os = "windows")]
        ShellType::Windows => todo!("Not implemented yet"),
        #[cfg(target_os = "windows")]
        ShellType::Cmd => todo!("Not implemented yet"),
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
    _exe_path: std::path::PathBuf,
    _print_full_init: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}

const POSIX_SHELL_WRAPPER_TEMPLATE: &str = include_str!("../../templates/posix.sh");

fn generate_posix_shell_wrapper(shell_type: &str, binary_path: &str) -> String {
    let marker_length = SHELL_MARK.len();

    POSIX_SHELL_WRAPPER_TEMPLATE
        .replace("{{SHELL_TYPE}}", shell_type)
        .replace("{{BINARY_PATH}}", binary_path)
        .replace("{{SHELL_CMD_MARKER}}", SHELL_MARK)
        .replace("{{MARKER_LENGTH}}", &marker_length.to_string())
}
