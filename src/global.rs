use std::sync::LazyLock;

use rand::{RngExt, distr::Alphanumeric};

pub static PEER_ID: LazyLock<[u8; 20]> = LazyLock::new(|| {
    let mut peer_id = [0u8; 20];

    peer_id[..8].copy_from_slice(b"-RS0001-");

    let random: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();

    peer_id[8..].copy_from_slice(random.as_bytes());

    peer_id
});
