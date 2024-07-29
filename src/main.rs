mod manual_parser;
mod torrent_parser;
mod tracker_parser;
mod handshake_parser;

extern crate core;

use crate::torrent_parser::{decode_torrent_file, TorrentFile};
use crate::tracker_parser::{decode_tracker_data, Tracker};
use crate::handshake_parser::{Handshake};
use std::env;
use std::fs::read;
use std::str;

enum Command {
    Decode,
    Info,
    Peers,
    Handshake,
    Unknown,
}

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "decode" => Self::Decode,
            "info" => Self::Info,
            "peers" => Self::Peers,
            "handshake" => Self::Handshake,
            _ => Self::Unknown,
        }
    }
}

fn get_torrent_info(file_name: &str) -> TorrentFile {
    let data = read(file_name).expect("read file");

    decode_torrent_file(&data)
}

fn url_encoding(url_part: &str) -> String {
    let encoded_url = url_part
        .chars()
        .collect::<Vec<char>>()
        .chunks(2)
        .map(|chunk| {
            let hex_value = chunk.iter().collect::<String>();
            match hex_value.as_str() {
                "4c" => "L".to_string(),
                "54" => "T".to_string(),
                "68" => "h".to_string(),
                "71" => "q".to_string(),
                _ => format!("%{}", hex_value),
            }
        })
        .collect::<Vec<String>>()
        .join("");

    encoded_url
}

async fn get_tracker(file_name: &str) -> Tracker {
    let torrent = get_torrent_info(file_name);
    let url = torrent.announce.clone().unwrap();
    let file_length = torrent.info.length.unwrap();
    let peer_id = "00112233445566778899";
    let port = "6881";
    let info_hash = url_encoding(&torrent.info.get_info_hash());

    let formated_url = format!(
        "{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact={}",
        url, info_hash, peer_id, port, 0, 0, file_length, 1
    );

    let body = reqwest::get(formated_url)
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();

    decode_tracker_data(body.as_ref())
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let command = Command::from(args[1].as_str());

    match command {
        Command::Decode => {
            let encoded_value = &args[2];
            let decoded_value = manual_parser::decode_bencoded_value(encoded_value);
            println!("{}", decoded_value);
        }
        Command::Info => {
            let torrent = get_torrent_info(&args[2]);

            torrent.print_all();
        }
        Command::Peers => {
            let tracker = get_tracker(&args[2]).await;

            tracker.print_peers_ips();
        }
        Command::Handshake => {
            // 165.232.33.77:51498
            // 178.62.82.89:51448
            // 178.62.85.20:51489
            let ip_data = &args[3];
            let torrent = get_torrent_info(&args[2]);
            let mut handshake = Handshake::new(torrent.info.get_info_hash(), [0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9]);

            let peer_id = handshake.fetch_peer_id(ip_data).await;

            // let len: u8 = 19;
            // let mut protocol_name = "BitTorrent protocol".to_owned().into_bytes();
            // let reserved = [0_u8; 8];
            // let mut info_hash = hex::decode(torrent.info.get_info_hash()).expect("decode info hash");
            // let peer_id: [u8; 20] = [0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9];
            //
            // println!("info_hash encoded: {}", torrent.info.get_info_hash());
            // println!("info_hash decoded: {:?}", &info_hash);
            //
            // let mut handshake: Vec<u8> = vec!();
            // handshake.push(len);
            // handshake.append(&mut protocol_name);
            // handshake.extend(reserved);
            // handshake.append(&mut info_hash);
            // handshake.extend(peer_id);
            //
            // println!("handshake len: {}", &handshake.as_slice().len());
            //
            // let mut stream = TcpStream::connect(ip_data).await.unwrap();
            //
            // stream.write_all(handshake.as_slice()).await.unwrap();
            //
            // let mut buffer = vec![0; 68];
            // stream.read_exact(&mut buffer).await.unwrap();
            //
            // let peer_id = &buffer[47..];
            //
            // println!("buffer: {:?}", peer_id);
            println!("Peer ID: {}", peer_id);
        }
        Command::Unknown => {
            println!("unknown command: {}", args[1])
        }
    }
}
