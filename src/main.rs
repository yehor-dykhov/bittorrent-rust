mod manual_parser;
mod torrent_parser;
mod tracker_parser;

extern crate core;

use crate::torrent_parser::{decode_torrent_file, TorrentFile};
use crate::tracker_parser::decode_tracker_data;
use std::env;
use std::fs::read;

enum Command {
    Decode,
    Info,
    Peers,
    Unknown,
}

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "decode" => Self::Decode,
            "info" => Self::Info,
            "peers" => Self::Peers,
            _ => Self::Unknown,
        }
    }
}

fn read_torrent_file(file_name: &str) -> TorrentFile {
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
            let torrent = read_torrent_file(&args[2]);

            torrent.print_all();
        }
        Command::Peers => {
            let torrent = read_torrent_file(&args[2]);
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

            let tracker = decode_tracker_data(body.as_ref());

            tracker.print_peers_ips();
        }
        Command::Unknown => {
            println!("unknown command: {}", args[1])
        }
    }
}
