use crate::global::PEER_ID;
use crate::{magnet, torrent};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Debug, thiserror::Error)]
pub enum HandshakeError {
    #[error(transparent)]
    ParseError(#[from] magnet::parse::Error),

    #[error(transparent)]
    PeerError(#[from] torrent::peers::PeerError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    SliceConversion(#[from] std::array::TryFromSliceError),

    #[error("no peers available")]
    NoPeers,

    #[error("invalid handshake response")]
    InvalidResponse,
}

pub async fn handshake(url: &str) -> Result<(), HandshakeError> {
    let info = magnet::parse(url)?;
    let peer_response =
        torrent::peers(&info.tracker_url, &info.info_hash_bytes, info.length).await?;

    let peer = peer_response.peers.first().ok_or(HandshakeError::NoPeers)?;

    let mut stream = TcpStream::connect(peer).await?;

    // build handshake: pstrlen + pstr + reserved + info_hash + peer_id
    let mut handshake = Vec::with_capacity(68);
    handshake.push(19);
    handshake.extend_from_slice(b"BitTorrent protocol");
    // reserved 20 bytes: bit 20 set (extension protocol support)
    handshake.extend_from_slice(&[0, 0, 0, 0, 0, 16, 0, 0]);
    handshake.extend_from_slice(&info.info_hash_bytes);
    handshake.extend_from_slice(&PEER_ID[..]);
    stream.write_all(&handshake).await?;

    let mut response = [0; 68];
    stream.read_exact(&mut response).await?;

    if !torrent::handshake::valid_response(&response, &info.info_hash_bytes) {
        return Err(HandshakeError::InvalidResponse);
    }

    let peer_id: [u8; 20] = response[48..68].try_into()?;
    println!("Peer ID: {}", hex::encode(peer_id));

    Ok(())
}
