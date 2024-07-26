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

impl Info {
    pub fn get_info_hash(&self) -> String {
        hex::encode(Sha1::digest(serde_bencode::to_bytes::<Info>(self).unwrap()))
    }

    pub fn get_piece_hashes(&self) -> Vec<String> {
        self.pieces.to_vec().chunks(20).map(hex::encode).collect()
    }
}

impl TorrentFile {
    pub fn print_all(&self) {
        let piece_hashes: Vec<String> = self.info.get_piece_hashes();
        let info_hash = &self.info.get_info_hash();

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
