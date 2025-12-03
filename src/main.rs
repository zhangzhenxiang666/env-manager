use env_manage::{cli::Cli, handles::run, utils::display};

fn main() {
    let cli = Cli::parse_args();
    if let Err(e) = run(cli) {
        display::show_error(&e.to_string());
        std::process::exit(1);
    }
}
