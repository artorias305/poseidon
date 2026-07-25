use crate::torrent::{self, download_piece};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    InfoError(#[from] torrent::info::InfoError),

    #[error(transparent)]
    DownloadPieceError(#[from] torrent::download_piece::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub async fn download(out_file: &str, file: &str, allow_hash_mismatch: bool) -> Result<(), Error> {
    let (mut stream, info, bitfield) = download_piece::connect_to_peer(file).await?;
    let mut file_data = Vec::with_capacity(info.length);

    for i in 0..info.num_pieces {
        let piece =
            download_piece::download_piece_data(&mut stream, &info, &bitfield, i, allow_hash_mismatch).await?;
        file_data.extend_from_slice(&piece);
    }

    std::fs::write(out_file, &file_data)?;
    Ok(())
}
