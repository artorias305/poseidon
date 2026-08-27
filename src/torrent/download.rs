use indicatif::{ProgressBar, ProgressStyle};

use crate::torrent::{self, download_piece};

const MAX_RETRIES: usize = 5;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    InfoError(#[from] torrent::info::InfoError),

    #[error(transparent)]
    DownloadPieceError(#[from] torrent::download_piece::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    PeerError(#[from] torrent::peers::PeerError),
}

pub async fn download(out_file: &str, file: &str, allow_hash_mismatch: bool) -> Result<(), Error> {
    let info = torrent::info(file)?;
    let all_peers = torrent::peers(&info.tracker_url, &info.info_hash_bytes, info.length).await?.peers;

    let mut file_data: Vec<Option<Vec<u8>>> = vec![None; info.num_pieces];
    let mut peer_idx = 0;

    let bar = ProgressBar::new(info.num_pieces as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} pieces ({eta})",
        )
        .expect("the progress bar template is valid"),
    );
    bar.enable_steady_tick(std::time::Duration::from_millis(100));
    let mut failed_pieces = 0;

    for i in 0..info.num_pieces {
        let mut success = false;

        for _attempt in 0..MAX_RETRIES {
            let peer = &all_peers[peer_idx % all_peers.len()];

            let (mut stream, bitfield) =
                match download_piece::connect_to_peer_with_peer(file, peer).await {
                    Ok(s) => s,
                    Err(_) => {
                        peer_idx += 1;
                        continue;
                    }
                };

            match download_piece::download_piece_data(
                &mut stream,
                &info,
                &bitfield,
                i,
                allow_hash_mismatch,
            )
            .await
            {
                Ok(piece) => {
                    file_data[i] = Some(piece);
                    success = true;
                    bar.inc(1);
                    break;
                }
                Err(_) => {
                    peer_idx += 1;
                    continue;
                }
            }
        }

        if !success {
            failed_pieces += 1;
            eprintln!("Failed to download piece {}", i);
        }
    }

    let mut final_data = Vec::with_capacity(info.length);
    for piece in file_data.into_iter().flatten() {
        final_data.extend_from_slice(&piece);
    }
    if failed_pieces == 0 {
        bar.finish_with_message("download complete");
    } else {
        bar.finish_with_message(format!("{} piece(s) failed", failed_pieces));
    }

    std::fs::write(out_file, &final_data)?;
    Ok(())
}
