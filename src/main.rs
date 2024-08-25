use std::{env, error::Error, path::PathBuf};

mod decompile;
mod types;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <classfile>", args[0]);
        // todo: return error
        return Ok(());
    }

    let path = PathBuf::from(&args[1]);
    if !path.exists() {
        todo!();
    }

    let mut file = std::fs::File::open(path)?;
    let mut dec = decompile::Decompile::new(&mut file);
    dec.decompile()?;

    Ok(())
}
