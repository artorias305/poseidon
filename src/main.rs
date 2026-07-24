mod bencode;
mod cli;
mod torrent;

use clap::Parser;
use cli::{Args, Commands};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Decode { encoded_value } => {
            let decoded_value = bencode::decode(encoded_value.as_bytes())?;
            dbg!(decoded_value);
        }
        Commands::Info { torrent_file } => {
            let info = torrent::info(&torrent_file);
            info.print();
        }
        Commands::Peers { torrent_file } => {
            let peer_response = torrent::peers(&torrent_file).await?;
            for peer in peer_response.peers {
                println!("{}", peer);
            }
        }
        Commands::Handshake { torrent_file, peer } => {
            println!("Torrent: {:?}", torrent_file);
            println!("Peer: {:?}", peer);
        }
    }

    Ok(())
}
