use clap::Parser;
use std::path::PathBuf;

mod decompile;
mod error;
mod types;

#[derive(Debug, Parser)]
struct Cli {
    file: PathBuf,
}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    let mut dec = decompile::Decompile::new(args.file)
        .map_err(|e| eprintln!("{}", e))
        .unwrap();

    if let Err(e) = dec.decompile() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
