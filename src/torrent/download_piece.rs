use sha1::Digest;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::torrent;

const BLOCK_SIZE: usize = 16 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    PeerError(#[from] torrent::peers::PeerError),

    #[error("no peers available")]
    NoPeersAvailable,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    HandshakeError(#[from] torrent::handshake::HandshakeError),

    #[error(transparent)]
    InfoError(#[from] torrent::info::InfoError),

    #[error("hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("peer does not have piece {0}")]
    PeerMissingPiece(usize),
}

pub async fn connect_to_peer(
    file: &str,
) -> Result<(TcpStream, torrent::info::TorrentInfo), Error> {
    let peer = torrent::peers(file)
        .await?
        .peers
        .first()
        .ok_or(Error::NoPeersAvailable)?
        .clone();

    let (stream, _handshake) = torrent::handshake(file, peer).await?;
    let info = torrent::info(file)?;
    Ok((stream, info))
}

pub async fn download_piece_data(
    stream: &mut TcpStream,
    info: &torrent::info::TorrentInfo,
    piece_index: usize,
    allow_hash_mismatch: bool,
) -> Result<Vec<u8>, Error> {
    let piece_length = info.piece_length;
    let num_blocks = (piece_length + BLOCK_SIZE - 1) / BLOCK_SIZE;

    let len = stream.read_u32().await?;
    let id = stream.read_u8().await?;

    assert_eq!(id, 5); // id for `bitfield` is 5

    let mut bitfield = vec![0u8; (len - 1) as usize];
    stream.read_exact(&mut bitfield).await?;

    let byte_index = piece_index / 8;
    let bit_offset = 7 - (piece_index % 8);
    if byte_index >= bitfield.len() || (bitfield[byte_index] >> bit_offset) & 1 == 0 {
        return Err(Error::PeerMissingPiece(piece_index));
    }

    let interested = [0, 0, 0, 1, 2];
    stream.write_all(&interested).await?;

    let _len = stream.read_u32().await?;
    let id = stream.read_u8().await?;

    assert_eq!(id, 1); // id for `unchoke` is 1

    let mut pieces: Vec<Vec<u8>> = Vec::with_capacity(num_blocks);

    for i in 0..num_blocks {
        let begin = i * BLOCK_SIZE;
        let length = std::cmp::min(BLOCK_SIZE, piece_length - begin);

        // request message: [4-byte length][1-byte id=6][4-byte index][4-byte begin][4-byte length]
        let mut request = Vec::with_capacity(17);
        request.extend_from_slice(&(13u32).to_be_bytes()); // payload length = 13
        request.push(6); // message id for request
        request.extend_from_slice(&(piece_index as u32).to_be_bytes());
        request.extend_from_slice(&(begin as u32).to_be_bytes());
        request.extend_from_slice(&(length as u32).to_be_bytes());

        stream.write_all(&request).await?;

        let len = stream.read_u32().await?;
        let id = stream.read_u8().await?;

        assert_eq!(id, 7); // id for `piece` is 7

        let mut block = vec![0u8; (len - 1) as usize];
        stream.read_exact(&mut block).await?;

        pieces.push(block);
    }

    let mut piece_data = Vec::with_capacity(piece_length);
    for block in &pieces {
        piece_data.extend_from_slice(block);
    }

    let hash = sha1::Sha1::digest(&piece_data);
    let hash_hex = hex::encode(hash);
    if hash_hex != info.piece_hashes[piece_index] && !allow_hash_mismatch {
        return Err(Error::HashMismatch {
            expected: info.piece_hashes[piece_index].clone(),
            actual: hash_hex,
        });
    }

    Ok(piece_data)
}

pub async fn download_piece(
    out_file: &str,
    file: &str,
    piece_index: usize,
    allow_hash_mismatch: bool,
) -> Result<(), Error> {
    let (mut stream, info) = connect_to_peer(file).await?;
    let piece_data = download_piece_data(&mut stream, &info, piece_index, allow_hash_mismatch).await?;
    std::fs::write(out_file, &piece_data)?;
    Ok(())
}
