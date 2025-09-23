use tagit::cli::run_cli;
use std::process;
fn main() {
    if let Err(e) = run_cli() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

