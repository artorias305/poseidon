use std::collections::BTreeMap;

use crate::bencode::BencodeValue;
use crate::global::PEER_ID;
use crate::{bencode, magnet, torrent};
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

    #[error(transparent)]
    DecodeError(#[from] bencode::DecodeError),
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
    handshake.extend_from_slice(&[0x10, 0, 0, 0, 0, 0, 0, 0]);
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

    let len = stream.read_u32().await?;
    let id = stream.read_u8().await?;
    assert_eq!(id, 5);

    let mut bitfield = vec![0u8; (len - 1) as usize];
    stream.read_exact(&mut bitfield).await?;

    let peer_supports_extensions = response[20] & 0x10 != 0;
    dbg!(peer_supports_extensions);

    if peer_supports_extensions {
        // payload is of the format [1-byte extension message id] [bencoded dictionary]
        let mut payload: Vec<u8> = Vec::new();
        payload.push(0); // id 0 for extension handshake

        let mut m_value: BTreeMap<String, BencodeValue> = BTreeMap::new();
        m_value.insert("ut_metadata".to_string(), BencodeValue::Int(1));
        m_value.insert("ut_pex".to_string(), BencodeValue::Int(2));

        let mut map: BTreeMap<String, BencodeValue> = BTreeMap::new();
        map.insert("m".to_string(), BencodeValue::Map(m_value));

        payload.extend_from_slice(&bencode::encode(&BencodeValue::Map(map)));

        let msg_len = (1 + payload.len()) as u32;
        stream.write_all(&msg_len.to_be_bytes()).await?;
        stream.write_u8(20).await?; // extended message id
        stream.write_all(&payload).await?;

        let len = stream.read_u32().await?;
        let id = stream.read_u8().await?;
        assert_eq!(id, 20); // extended message

        let mut ext_payload = vec![0u8; (len - 1) as usize];
        stream.read_exact(&mut ext_payload).await?;

        assert_eq!(ext_payload[0], 0); // first byte is extension msg id (0 = ext handshake)

        if let BencodeValue::Map(map) = bencode::decode(&ext_payload[1..])? {
            if let Some(BencodeValue::Map(m)) = map.get("m") {
                println!("Peer extensions: {:?}", m);
            }
        }
    }

    Ok(())
}
