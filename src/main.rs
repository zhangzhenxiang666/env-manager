use env_manage::cli::Cli;
use env_manage::handles::run;
fn main() {
    let cli = Cli::parse_args();
    if let Err(e) = run(cli) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
