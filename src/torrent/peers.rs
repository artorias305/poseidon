use crate::torrent::info;

#[derive(Debug, thiserror::Error)]
pub enum PeerError {
    #[error("failed to parse URL")]
    ParseError,
    #[error("request failed")]
    GetError,
}

pub async fn peers(file: &str) -> Result<(), PeerError> {
    let file_data_as_bytes = std::fs::read(file).unwrap();

    let info = info(file);

    let info_hash_enc = percent_encode(&info.info_hash_bytes);
    let url_str = format!(
        "{}?info_hash={}&peer_id={}&port=6881&uploaded=0&downloaded=0&left={}&compact=1",
        info.tracker_url,
        info_hash_enc,
        "thisisthecoolpeeridd",
        file_data_as_bytes.len()
    );
    let url = reqwest::Url::parse(&url_str).map_err(|_| PeerError::ParseError)?;

    let response = reqwest::get(url).await.map_err(|_| PeerError::GetError)?;
    dbg!(response.text().await.unwrap());

    Ok(())
}

fn percent_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("%{:02X}", b)).collect()
}
