use crate::{
    bencode::{self, BencodeValue},
    torrent::info,
};

#[derive(Debug, thiserror::Error)]
pub enum PeerError {
    #[error("failed to parse URL")]
    ParseError,
    #[error("request failed")]
    GetError,
}

#[derive(Default, Debug)]
pub struct PeerResponse {
    interval: usize,
    pub peers: Vec<String>,
}

impl PeerResponse {
    fn from_bytes(raw: &[u8]) -> Result<Self, bencode::DecodeError> {
        let mut response = PeerResponse::default();

        if let bencode::BencodeValue::Map(map) = bencode::decode(raw)? {
            if let Some(BencodeValue::Int(n)) = map.get("interval") {
                response.interval = *n as usize;
            }

            if let Some(BencodeValue::String(peers)) = map.get("peers") {
                let peers: Vec<String> = peers
                    .chunks(6)
                    .map(|chunk| {
                        format!(
                            "{}.{}.{}.{}:{}",
                            chunk[0],
                            chunk[1],
                            chunk[2],
                            chunk[3],
                            u16::from_be_bytes([chunk[4], chunk[5]])
                        )
                    })
                    .collect();
                response.peers = peers;
            }
        }

        Ok(response)
    }
}

pub async fn peers(file: &str) -> Result<PeerResponse, PeerError> {
    let info = info(file);

    let info_hash_enc = percent_encode(&info.info_hash_bytes);
    let url_str = format!(
        "{}?info_hash={}&peer_id={}&port=6881&uploaded=0&downloaded=0&left={}&compact=1",
        info.tracker_url, info_hash_enc, "thisisthecoolpeeridd", info.length
    );
    let url = reqwest::Url::parse(&url_str).map_err(|_| PeerError::ParseError)?;

    let response = reqwest::get(url).await.map_err(|_| PeerError::GetError)?;
    let raw = response.bytes().await.map_err(|_| PeerError::GetError)?;
    let response = PeerResponse::from_bytes(&raw).map_err(|_| PeerError::ParseError)?;

    Ok(response)
}

fn percent_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("%{:02X}", b)).collect()
}
