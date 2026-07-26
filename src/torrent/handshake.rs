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
    #[error("invalid peer")]
    InvalidPeer,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    InfoError(#[from] torrent::info::InfoError),

    #[error(transparent)]
    SliceConversion(#[from] std::array::TryFromSliceError),

    #[error(transparent)]
    PeerError(#[from] torrent::peers::PeerError),

    #[error("invalid handshake response")]
    InvalidResponse,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Handshake {
    pub reserved: [u8; 8],
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

pub async fn handshake(
    file: &str,
    peer: SocketAddr,
) -> Result<(TcpStream, Handshake), HandshakeError> {
    let info = torrent::info(file)?;
    let peers = peers(&info.tracker_url, &info.info_hash_bytes, info.length).await?;

    if !peers.peers.contains(&peer) {
        return Err(HandshakeError::InvalidPeer);
    }

    let mut stream = TcpStream::connect(peer).await?;

    let mut handshake = Vec::with_capacity(68);
    handshake.push(19);
    handshake.extend_from_slice(b"BitTorrent protocol");
    handshake.extend_from_slice(&[0u8; 8]);
    handshake.extend_from_slice(&info.info_hash_bytes);
    handshake.extend_from_slice(&PEER_ID[..]);
    stream.write_all(&handshake).await?;

    let mut response = [0u8; 68];
    stream.read_exact(&mut response).await?;

    if !valid_response(&response, &info.info_hash_bytes) {
        return Err(HandshakeError::InvalidResponse);
    }

    // NOTE: The errors here should never occur, just avoiding unwrap
    let reserved: [u8; 8] = response[20..28].try_into()?;
    let info_hash: [u8; 20] = response[28..48].try_into()?;
    let peer_id: [u8; 20] = response[48..68].try_into()?;

    Ok((
        stream,
        Handshake {
            reserved,
            info_hash,
            peer_id,
        },
    ))
}

pub fn valid_response(response: &[u8], info_hash_bytes: &[u8]) -> bool {
    if response[0] != 19 {
        return false;
    }

    if &response[1..20] != b"BitTorrent protocol" {
        return false;
    }

    if &response[28..48] != info_hash_bytes {
        return false;
    }

    true
}
