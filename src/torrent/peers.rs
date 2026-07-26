use std::net::{Ipv4Addr, SocketAddr};

use crate::{
    bencode::{self, BencodeValue},
    global::PEER_ID,
    torrent::{self, info, utils},
};

#[derive(Debug, thiserror::Error)]
pub enum PeerError {
    #[error(transparent)]
    Url(#[from] url::ParseError),

    #[error(transparent)]
    Decode(#[from] bencode::DecodeError),

    #[error(transparent)]
    GetError(#[from] reqwest::Error),

    #[error(transparent)]
    InfoError(#[from] torrent::info::InfoError),

    #[error("tracker returned an invalid compact peer list")]
    InvalidPeerList,
}

#[derive(Default, Debug)]
pub struct PeerResponse {
    interval: usize,
    pub peers: Vec<SocketAddr>,
}

impl PeerResponse {
    fn from_bytes(raw: &[u8]) -> Result<Self, PeerError> {
        let mut response = PeerResponse::default();

        if let bencode::BencodeValue::Map(map) = bencode::decode(raw)? {
            if let Some(BencodeValue::Int(n)) = map.get("interval") {
                response.interval = *n as usize;
            }

            if let Some(BencodeValue::String(peers)) = map.get("peers") {
                if peers.len() % 6 != 0 {
                    return Err(PeerError::InvalidPeerList);
                }

                let peers: Vec<SocketAddr> = peers
                    .chunks(6)
                    .map(|chunk| {
                        (
                            Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]),
                            u16::from_be_bytes([chunk[4], chunk[5]]),
                        )
                            .into()
                    })
                    .collect();

                response.peers = peers;
            }
        }

        Ok(response)
    }
}

pub async fn peers(tracker_url: &str, info_hash_bytes: &[u8], length: usize) -> Result<PeerResponse, PeerError> {
    let info_hash_enc = utils::percent_encode(info_hash_bytes);
    let peer_id_enc = utils::percent_encode(&PEER_ID[..]);
    let url_str = format!(
        "{}?info_hash={}&peer_id={}&port=6881&uploaded=0&downloaded=0&left={}&compact=1",
        tracker_url, info_hash_enc, peer_id_enc, length
    );
    let url = reqwest::Url::parse(&url_str)?;

    let response = reqwest::get(url).await?;
    let raw = response.bytes().await?;
    let response = PeerResponse::from_bytes(&raw)?;

    Ok(response)
}
