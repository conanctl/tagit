use pathbrain::cli::run_cli;
use std::process;

fn main() {
    if let Err(e) = run_cli() {
        eprintln!("{}", e.user_friendly_message());
        process::exit(1);
    }
}

