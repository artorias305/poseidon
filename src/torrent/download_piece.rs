use sha1::Digest;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
}

/// struct representing a piece message
#[allow(dead_code)]
struct PieceMessage {
    /// The zero-based piece index
    index: usize,

    /// The zero-based byte offset within the piece
    begin: usize,

    /// The data for the piece, usually 2^14 bytes long
    block: Vec<u8>,
}

pub async fn download_piece(
    out_file: &str,
    file: &str,
    piece_index: usize,
    allow_hash_mismatch: bool,
) -> Result<(), Error> {
    let peer = torrent::peers(file)
        .await?
        .peers
        .first()
        .ok_or(Error::NoPeersAvailable)?
        .clone();

    let info = torrent::info(file)?;
    let piece_length = info.piece_length;
    let num_blocks = (piece_length + BLOCK_SIZE - 1) / BLOCK_SIZE;

    let (mut stream, _handshake) = torrent::handshake(file, peer).await?;

    let len = stream.read_u32().await?;
    let id = stream.read_u8().await?;

    assert_eq!(id, 5); // id for `bitfield` is 5

    let mut bitfield = vec![0u8; (len - 1) as usize];
    stream.read_exact(&mut bitfield).await?;

    let interested = [0, 0, 0, 1, 2];
    stream.write_all(&interested).await?;

    let _len = stream.read_u32().await?;
    let id = stream.read_u8().await?;

    assert_eq!(id, 1); // id for `unchoke` is 1

    let mut pieces: Vec<PieceMessage> = Vec::with_capacity(num_blocks);

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

        let mut piece_message = vec![0u8; (len - 1) as usize];
        stream.read_exact(&mut piece_message).await?;

        let piece = PieceMessage {
            index: piece_index,
            begin,
            block: piece_message,
        };

        pieces.push(piece);
    }

    let mut piece_data = Vec::with_capacity(piece_length);
    for piece in &pieces {
        piece_data.extend_from_slice(&piece.block);
    }

    let hash = sha1::Sha1::digest(&piece_data);
    let hash_hex = hex::encode(hash);
    if hash_hex != info.piece_hashes[piece_index] && !allow_hash_mismatch {
        return Err(Error::HashMismatch {
            expected: info.piece_hashes[piece_index].clone(),
            actual: hash_hex,
        });
    }

    std::fs::write(out_file, &piece_data)?;

    Ok(())
}
