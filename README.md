# Poseidon 🌊

A BitTorrent client written in Rust.

## Installation

```bash
git clone https://github.com/artorias305/poseidon.git
cd poseidon
cargo build --release
```

## Usage

```bash
# Decode a bencoded value
poseidon decode "d3:cow4:moo01:4spam01:e"

# Show torrent file info
poseidon info <torrent_file>

# List peers
poseidon peers <torrent_file>

# Handshake with a peer
poseidon handshake <torrent_file> <peer_ip:peer_port>

# Download one piece
poseidon download-piece -o <output_file> <torrent_file> <piece_index>

# Download a full file
poseidon download -o <output_file> <torrent_file>
```

Add `--allow-hash-mismatch` to skip SHA-1 verification when downloading pieces.

## Example

```bash
poseidon info ubuntu.torrent
poseidon download -o ubuntu.iso ubuntu.torrent
poseidon download-piece -o piece0.bin ubuntu.torrent 0
```

## To-Do

- [ ] Support for magnet links
