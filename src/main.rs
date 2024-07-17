mod manual_parser;
mod parser;

extern crate core;

use sha1::{Digest, Sha1};
use std::env;
use std::fs::read;
use crate::parser::{decode_torrent_file, Info};

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = manual_parser::decode_bencoded_value(encoded_value);
        println!("{}", decoded_value);
    } else if command == "info" {
        let file_name = &args[2];
        let data = read(file_name).expect("read file");
        let torrent = decode_torrent_file(&data);

        println!("Tracker URL: {}", torrent.announce.unwrap());
        println!("Length: {}", torrent.info.length.unwrap());
        println!(
            "Info Hash: {}",
            hex::encode(Sha1::digest(serde_bencode::to_bytes::<Info>(&torrent.info).unwrap()))
        );
    } else {
        println!("unknown command: {}", args[1])
    }
}
