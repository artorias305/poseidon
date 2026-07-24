mod bencode;
mod cli;

use clap::Parser;
use cli::{Args, Commands};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Commands::Decode { encoded_value } => {
            let decoded_value = bencode::decode(&encoded_value)?;
            dbg!(decoded_value);
        }
    }

    Ok(())
}
