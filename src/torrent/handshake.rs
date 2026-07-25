use std::net::SocketAddr;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{
    global::PEER_ID,
    torrent::{self, peers},
};

#[derive(Debug, thiserror::Error)]
pub enum HandshakeError {
    #[error("peers not found")]
    PeersNotFound,
    #[error("invalid peer")]
    InvalidPeer,
    #[error("connection error")]
    ConnectionError,
    #[error("write error")]
    WriteError,
    #[error("read error")]
    ReadError,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Handshake {
    pub reserved: [u8; 8],
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

pub async fn handshake(file: &str, peer: SocketAddr) -> Result<Handshake, HandshakeError> {
    let peers = peers(file)
        .await
        .map_err(|_| HandshakeError::PeersNotFound)?;
    let info = torrent::info(file);

    if !peers.peers.contains(&peer.to_string()) {
        return Err(HandshakeError::InvalidPeer);
    }

    let mut stream = TcpStream::connect(peer)
        .await
        .map_err(|_| HandshakeError::ConnectionError)?;

    let mut handshake = Vec::with_capacity(68);
    handshake.push(19);
    handshake.extend_from_slice(b"BitTorrent protocol");
    handshake.extend_from_slice(&[0u8; 8]);
    handshake.extend_from_slice(&info.info_hash_bytes);
    handshake.extend_from_slice(&PEER_ID[..]);
    stream
        .write_all(&handshake)
        .await
        .map_err(|_| HandshakeError::WriteError)?;

    let mut response = [0u8; 68];
    stream
        .read_exact(&mut response)
        .await
        .map_err(|_| HandshakeError::ReadError)?;

    if !valid_response(&response, &info.info_hash_bytes) {
        return Err(HandshakeError::ConnectionError);
    }

    let reserved: [u8; 8] = response[20..28].try_into().unwrap();
    let info_hash: [u8; 20] = response[28..48].try_into().unwrap();
    let peer_id: [u8; 20] = response[48..68].try_into().unwrap();

    Ok(Handshake {
        reserved,
        info_hash,
        peer_id,
    })
}

fn valid_response(response: &[u8], info_hash_bytes: &[u8]) -> bool {
    if response[0] != 19 {
        return false;
    }

    if &response[1..20] != b"BitTorrent protocol" {
        return false;
    }

    if &response[28..48] != info_hash_bytes {
        return false;
    }

    return true;
}
