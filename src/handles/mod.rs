use crate::cli::Cli;
use crate::cli::Commands::{Activate, Deactivate, Global, Init, Profile, Status, Ui};

mod activate;
mod deactivate;
mod global;
mod init;
mod profile;
mod status;
mod ui;

pub fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Init => init::handle(),
        Profile(profile_commands) => profile::handle(profile_commands),
        Activate { items } => activate::handle(items),
        Deactivate { items } => deactivate::handle(items),
        Global(global_commands) => global::handle(global_commands),
        Status(status_args) => status::handle(status_args),
        Ui => ui::handle(),
    }
}
