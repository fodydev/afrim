use clafrica::{api, prelude::Config, run};
use std::{env, path::Path, process};

fn main() {
    let frontend = api::Console::default();

    let filename = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Configuration file required");
        process::exit(1);
    });

    let conf = Config::from_file(Path::new(&filename)).unwrap_or_else(|err| {
        eprintln!("Problem parsing config file: {err}");
        process::exit(1);
    });

    if let Err(e) = run(conf, frontend) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
