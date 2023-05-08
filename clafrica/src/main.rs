use clafrica::{api, run, Config};
use std::{env, process};

fn main() {
    let frontend = api::Console;

    let conf = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = run(conf, frontend) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
