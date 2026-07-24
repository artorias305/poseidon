use sha1::{Digest, Sha1};

use crate::bencode::{self, BencodeValue};

#[derive(Default)]
pub struct TorrentInfo {
    pub tracker_url: String,
    pub length: usize,
    pub info_hash: String,
    pub piece_length: usize,
    pub piece_hashes: Vec<String>,
}

impl TorrentInfo {
    pub fn print(&self) {
        println!("Tracker URL: {}", self.tracker_url);
        println!("Length: {}", self.length);
        println!("Info Hash: {}", self.info_hash);
        println!("Piece Length: {}", self.piece_length);
        println!("Piece Hashes:");
        for piece in self.piece_hashes.iter() {
            println!("{}", piece);
        }
    }
}

pub fn info(file: &str) -> TorrentInfo {
    let file_data_as_bytes = std::fs::read(file).unwrap();
    let decoded_file = bencode::decode(&file_data_as_bytes).unwrap();
    let mut torrent_info = TorrentInfo::default();

    if let BencodeValue::Map(top) = decoded_file {
        if let Some(BencodeValue::String(s)) = top.get("announce") {
            torrent_info.tracker_url = String::from_utf8_lossy(s).into_owned();
        }

        if let Some(info) = top.get("info") {
            if let BencodeValue::Map(info_map) = info {
                if let Some(BencodeValue::Int(n)) = info_map.get("length") {
                    torrent_info.length = *n as usize;
                }

                let encoded_info = bencode::encode(info);
                let mut hasher = Sha1::new();
                hasher.update(&encoded_info);
                let hash = hasher.finalize();
                torrent_info.info_hash = hex::encode(hash);

                if let Some(BencodeValue::Int(n)) = info_map.get("piece length") {
                    torrent_info.piece_length = *n as usize;
                }

                if let Some(BencodeValue::String(pieces)) = info_map.get("pieces") {
                    let pieces: Vec<String> = pieces.chunks(20).map(hex::encode).collect();
                    torrent_info.piece_hashes = pieces;
                }
            }
        }
    }
    torrent_info
}
