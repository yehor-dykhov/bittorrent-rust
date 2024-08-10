use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct Handshake {
    len: u8,
    protocol: Vec<u8>,
    info_hash: Vec<u8>,
    peer_id: [u8; 20],
    pub response_peer_id: Option<String>,
}

impl Handshake {
    pub fn new(info_hash: String, peer_id: [u8; 20]) -> Self {
        Handshake {
            len: 19,
            protocol: "BitTorrent protocol".to_owned().into_bytes(),
            info_hash: hex::decode(info_hash).expect("decoded info hash"),
            peer_id,
            response_peer_id: None,
        }
    }

    pub async fn handshake(&mut self, ip: &str) -> TcpStream {
        let mut handshake: Vec<u8> = vec![];
        handshake.push(self.len);
        handshake.extend(self.protocol.as_slice());
        handshake.extend([0_u8; 8]);
        handshake.extend(self.info_hash.as_slice());
        handshake.extend(self.peer_id);

        let mut stream = TcpStream::connect(ip).await.expect("Connection");

        stream.write_all(handshake.as_slice()).await.expect("Do handshake");

        let mut buffer = vec![0; 68];
        stream.read_exact(&mut buffer).await.expect("Read response of handshake");

        let response_peer_id = &buffer[48..];
        self.response_peer_id = Some(hex::encode(response_peer_id));

        stream
    }
}
