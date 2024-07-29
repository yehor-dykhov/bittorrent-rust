use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct Handshake {
    len: u8,
    protocol: Vec<u8>,
    info_hash: Vec<u8>,
    peer_id: [u8; 20],
}

impl Handshake {
    pub fn new(info_hash: String, peer_id: [u8; 20]) -> Self {
        Handshake { len: 19, protocol: "BitTorrent protocol".to_owned().into_bytes(), info_hash: hex::decode(info_hash).expect("decoded info hash"), peer_id }
    }

    pub async fn fetch_peer_id(&mut self, ip: &str) -> String {
        let mut handshake: Vec<u8> = vec!();
        handshake.push(self.len);
        handshake.append(&mut self.protocol);
        handshake.extend([0_u8; 8]);
        handshake.append(&mut self.info_hash);
        handshake.extend(self.peer_id);

        let mut stream = TcpStream::connect(ip).await.unwrap();

        stream.write_all(handshake.as_slice()).await.unwrap();

        let mut buffer = vec![0; 68];
        stream.read_exact(&mut buffer).await.unwrap();

        let response_peer_id = &buffer[48..];

        hex::encode(response_peer_id)
    }
}