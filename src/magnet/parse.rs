use std::collections::HashMap;

#[allow(dead_code)]
pub struct MagnetInfo {
    pub tracker_url: String,
    pub info_hash: String,
    pub info_hash_bytes: Vec<u8>,
    pub length: usize,
}

impl MagnetInfo {
    pub fn print(&self) {
        println!("Tracker URL: {}", self.tracker_url);
        println!("Info Hash: {}", self.info_hash);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ParseError(#[from] url::ParseError),

    #[error("missing tracker url")]
    MissingTrackerUrl,

    #[error("missing xt")]
    MissingXt,

    #[error("invalid xt")]
    InvalidXt,

    #[error(transparent)]
    HexDecodeError(#[from] hex::FromHexError),
}

pub fn parse(url: &str) -> Result<MagnetInfo, Error> {
    let parsed_url = url::Url::parse(url)?;
    let query: HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();

    let length = query
        .get("xl")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(usize::MAX);

    let tracker_url = query.get("tr").ok_or(Error::MissingTrackerUrl)?.clone();

    let xt = query.get("xt").ok_or(Error::MissingXt)?;

    let mut parts = xt.split(':');

    let info_hash = match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some("urn"), Some("btih"), Some(hash), None) => hash.to_string(),
        _ => return Err(Error::InvalidXt),
    };

    let info_hash_bytes = hex::decode(&info_hash)?;

    Ok(MagnetInfo {
        tracker_url,
        info_hash,
        info_hash_bytes,
        length,
    })
}
