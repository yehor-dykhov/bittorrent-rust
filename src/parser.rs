use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

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

pub fn decode_torrent_file (data: &[u8]) -> TorrentFile {
    serde_bencode::from_bytes(data).unwrap()
}