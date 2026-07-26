use std::net::SocketAddr;

use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    /// Decode a bencoded value
    Decode {
        /// Bencoded value
        encoded_value: String,
    },

    /// Get information about a torrent file
    Info {
        /// Path to the torrent file
        torrent_file: String,
    },

    /// Get the peers from a torrent file
    Peers {
        /// Path to the torrent file
        torrent_file: String,
    },

    /// Establish a TCP connection with a peer and complete a handshake
    Handshake {
        /// Path to the torrent file
        torrent_file: String,
        /// Peer address (e.g. 127.0.0.1:6881)
        #[arg(value_name = "peer_ip:peer_port")]
        peer: SocketAddr,
    },

    /// Download a piece
    DownloadPiece {
        /// Output file
        #[arg(short, long)]
        output: String,
        /// Path to the torrent file
        torrent_file: String,
        /// Piece index
        piece_index: usize,
        /// Skip piece hash verification
        #[arg(long)]
        allow_hash_mismatch: bool,
    },

    /// Download a file
    Download {
        #[arg(short, long)]
        /// Output file
        output: String,
        /// Path to the torrent file
        torrent_file: String,
        /// Skip piece hash verification
        #[arg(long)]
        allow_hash_mismatch: bool,
    },

    /// Parse a Magnet url
    MagnetParse {
        /// Magnet url to parse
        magnet_url: String,
    },

    /// Handshake a magnet url
    MagnetHandshake {
        /// Magnet url to contact
        magnet_url: String,
    },
}

#[derive(Parser)]
#[command(version, about = "CLI for torrent written in rust")]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}
