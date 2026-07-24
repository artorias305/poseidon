use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    Decode { encoded_value: String },
    Info { torrent_file: String },
    Peers { torrent_file: String },
}

#[derive(Parser)]
#[command(version, about = "CLI for torrent written in rust")]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}
