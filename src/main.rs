use std::{env, process};

use todos::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = Command::build(&args).unwrap_or_else(|err| {
        eprintln!("Command error: {err}");
        process::exit(1);
    });
    if let Err(err) = todos::run(command) {
        eprintln!("App error: {err}");
        process::exit(1);
    }

    process::exit(0);
}
