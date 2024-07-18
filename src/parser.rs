use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};

#[derive(Deserialize, Serialize, Debug)]
pub struct TorrentFile {
    pub announce: Option<String>,
    pub info: Info,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Info {
    pub length: Option<isize>,
    pub name: String,
    #[serde(rename(deserialize = "piece length", serialize = "piece length"))]
    pub piece_length: isize,
    pub pieces: ByteBuf,
}

impl TorrentFile {
    pub fn print(&self) {
        let piece_hashes: Vec<String> = self
            .info
            .pieces
            .to_vec()
            .chunks(20)
            .map(hex::encode)
            .collect();
        let info_hash = hex::encode(Sha1::digest(
            serde_bencode::to_bytes::<Info>(&self.info).unwrap(),
        ));

        println!("Tracker URL: {}", self.announce.clone().unwrap());
        println!("Length: {}", self.info.length.unwrap());
        println!("Info Hash: {}", info_hash);
        println!("Piece Length: {}", self.info.piece_length);
        println!("Piece Hashes:");
        for piece_hash in piece_hashes {
            println!("{}", piece_hash);
        }
    }
}

pub fn decode_torrent_file(data: &[u8]) -> TorrentFile {
    serde_bencode::from_bytes(data).unwrap()
}
