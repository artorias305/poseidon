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

    let params = [
        ("info_hash", info.info_hash),
        ("peer_id", "thisisthecoolpeeridd".to_string()),
        ("port", "6881".to_string()),
        ("uploaded", "0".to_string()),
        ("downloaded", "0".to_string()),
        ("left", file_data_as_bytes.len().to_string()),
        ("compact", "1".to_string()),
    ];

    let url = reqwest::Url::parse_with_params(&info.tracker_url, &params)
        .map_err(|_| PeerError::ParseError)?;

    let response = reqwest::get(url).await.map_err(|_| PeerError::GetError)?;
    dbg!(response);

    Ok(())
}
