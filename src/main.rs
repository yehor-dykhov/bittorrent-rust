mod manual_parser;
mod parser;

extern crate core;

use crate::parser::{decode_torrent_file};
use std::env;
use std::fs::read;

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

        torrent.print();
        // let piece_hashes: Vec<String> = torrent.info.pieces.to_vec().chunks(20).map(|chunk| hex::encode(Sha1::digest(chunk))).collect();
        //
        // println!("Tracker URL: {}", torrent.announce.unwrap());
        // println!("Length: {}", torrent.info.length.unwrap());
        // println!(
        //     "Info Hash: {}",
        //     hex::encode(Sha1::digest(serde_bencode::to_bytes::<Info>(&torrent.info).unwrap()))
        // );
        // println!("Piece Length: {}", torrent.info.piece_length);
        // println!("Piece Hashes:");
        // for piece_hash in piece_hashes {
        //     println!("{}", piece_hash);
        // }
    } else {
        println!("unknown command: {}", args[1])
    }
}
