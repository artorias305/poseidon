use sha1::{Digest, Sha1};

use crate::bencode::{self, BencodeValue};

pub fn info(file: &str) {
    let file_data_as_bytes = std::fs::read(file).unwrap();
    let decoded_file = bencode::decode(&file_data_as_bytes).unwrap();

    if let BencodeValue::Map(top) = decoded_file {
        if let Some(BencodeValue::String(s)) = top.get("announce") {
            println!("Tracker URL: {}", String::from_utf8_lossy(s));
        }

        if let Some(info) = top.get("info") {
            if let BencodeValue::Map(info_map) = info {
                if let Some(BencodeValue::Int(n)) = info_map.get("length") {
                    println!("Length: {}", n);
                }

                let encoded_info = bencode::encode(info);
                let mut hasher = Sha1::new();
                hasher.update(&encoded_info);
                let hash = hasher.finalize();
                println!("Info Hash: {}", hex::encode(hash));

                if let Some(BencodeValue::Int(n)) = info_map.get("piece length") {
                    println!("Piece Length: {}", n);
                }

                if let Some(BencodeValue::String(pieces)) = info_map.get("pieces") {
                    let pieces: Vec<String> = pieces
                        .chunks(20)
                        .map(hex::encode)
                        .collect();

                    println!("Piece Hashes:");
                    for piece in pieces {
                        println!("{piece}");
                    }
                }
            }
        }
    }
}
