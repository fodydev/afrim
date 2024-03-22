use afrim::{frontend, run, Config};
use clap::Parser;
use std::process;

/// Afrim CLI.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the configuration file.
    config_file: std::path::PathBuf,

    /// Only verify if the configuration file is valid.
    #[arg(long, action)]
    check: bool,
}

fn main() {
    let args = Args::parse();
    let frontend = frontend::Console::default();

    let conf = Config::from_file(&args.config_file).unwrap_or_else(|err| {
        eprintln!("Problem with config file: {err:?}");
        process::exit(1);
    });

    if !args.check {
        run(conf, frontend).unwrap_or_else(|err| {
            eprintln!("Application error: {err:?}");
            process::exit(1);
        });
    }
}
