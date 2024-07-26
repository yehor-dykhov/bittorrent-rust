use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::str;

#[derive(Deserialize, Serialize, Debug)]
pub struct Tracker {
    pub interval: isize,
    pub peers: ByteBuf,
}

impl Tracker {
    pub fn get_peers_ips(&self) -> Vec<String> {
        let peers_chunks: Vec<(String, String)> = self
            .peers
            .to_vec()
            .chunks(6)
            .map(|chunk| {
                let ip_chunk = chunk[..4].iter().map(|n| n.to_string()).collect::<Vec<String>>();
                let port_chunk = &chunk[4..];
                let port = (port_chunk[0] as u16) << 8 | port_chunk[1] as u16;

                (ip_chunk.join("."), port.to_string())
            })
            .collect();

        peers_chunks
            .into_iter()
            .map(|(ip, port)| format!("{}:{}", ip, port))
            .collect()
    }

    pub fn print_peers_ips(&self) {
        for ip in self.get_peers_ips() {
            println!("{}", ip);
        }
    }
}

pub fn decode_tracker_data(data: &[u8]) -> Tracker {
    serde_bencode::from_bytes(data).unwrap()
}
