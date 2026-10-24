use std::env;
use std::process;

use rgrep::cli::Config;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(err) = rgrep::run(config) {
        eprintln!("Application error: {err}");
        process::exit(1);
    }
}
