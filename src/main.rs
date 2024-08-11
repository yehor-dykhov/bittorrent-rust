mod handshake_parser;
mod manual_parser;
mod torrent_parser;
mod tracker_parser;

extern crate core;

use crate::handshake_parser::Handshake;
use crate::torrent_parser::{decode_torrent_file, TorrentFile};
use crate::tracker_parser::{decode_tracker_data, Tracker};
use std::cmp::min;
use std::fs::read;
use std::{env, str};
use tokio::fs::write;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

enum Command {
    Decode,
    Info,
    Peers,
    Handshake,
    DownloadPiece,
    Download,
    Unknown,
}

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "decode" => Self::Decode,
            "info" => Self::Info,
            "peers" => Self::Peers,
            "handshake" => Self::Handshake,
            "download_piece" => Self::DownloadPiece,
            "download" => Self::Download,
            _ => Self::Unknown,
        }
    }
}

fn get_torrent_info(file_name: &str) -> TorrentFile {
    let data = read(file_name).expect("read file");

    decode_torrent_file(&data)
}

async fn save_pieces_to_file(path: &str, piece: &[u8]) {
    write(path, piece).await.expect("write to file");
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
    let file_length = torrent.info.length;
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

async fn read_message(stream: &mut TcpStream) -> (u8, Vec<u8>) {
    let mut prefix = [0u8; 4];

    stream
        .read_exact(&mut prefix)
        .await
        .expect("received prefix");

    let message_len = u32::from_be_bytes(prefix);
    let mut message = vec![0u8; message_len as usize];

    stream
        .read_exact(&mut message)
        .await
        .expect("received message");

    let message_id = message[0];
    let payload = message[1..].to_vec();

    (message_id, payload)
}

async fn write_message(stream: &mut TcpStream, id: u8, payload: Vec<u8>) {
    let len: u32 = (payload.len() + 1) as u32;
    let prefix_len = len.to_be_bytes();
    let mut message = vec![0u8; 0];

    message.extend_from_slice(&prefix_len);
    message.push(id);
    message.extend_from_slice(&payload);

    stream.write_all(&message).await.expect("send message");
}

async fn load_piece_block(stream: &mut TcpStream, index: u32, begin: u32, length: u32) -> Vec<u8> {
    let mut payload = vec![];

    payload.extend_from_slice(&index.to_be_bytes());
    payload.extend_from_slice(&begin.to_be_bytes());
    payload.extend_from_slice(&length.to_be_bytes());

    write_message(stream, 6, payload).await;

    let (_message_id, data) = read_message(stream).await;

    data[8..8 + length as usize].to_vec()
}

async fn download_piece(
    stream: &mut TcpStream,
    torrent_length: usize,
    piece_length: usize,
    piece_number: usize,
) -> Vec<u8> {
    let mut piece: Vec<u8> = vec![];
    let block_size = 16 * 1024;
    let mut begin = 0;
    let torrent_size_remain = torrent_length - piece_length * piece_number;
    let mut piece_size_remain = min(torrent_size_remain, piece_length);

    while piece_size_remain > 0 {
        let next_block_size = min(piece_size_remain, block_size);

        let block = load_piece_block(
            stream,
            piece_number as u32,
            begin as u32,
            next_block_size as u32,
        )
        .await;

        piece.extend_from_slice(&block);

        piece_size_remain -= next_block_size;
        begin += next_block_size;
    }

    piece
}

async fn init_download(torrent_file_name: &str) -> (TcpStream, Vec<String>, usize, usize) {
    let tracker = get_tracker(torrent_file_name).await;
    let torrent = get_torrent_info(torrent_file_name);
    let torrent_length = torrent.info.length;
    let piece_length = torrent.info.piece_length;

    let mut handshake = Handshake::new(
        torrent.info.get_info_hash(),
        [0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9],
    );
    let ips = tracker.get_peers_ips();
    let (ip, port) = ips.first().unwrap();

    let mut stream = handshake
        .handshake(format!("{}:{}", ip, port).as_str())
        .await;

    read_message(&mut stream).await;

    write_message(&mut stream, 2, vec![]).await;

    read_message(&mut stream).await;

    (
        stream,
        torrent.info.get_piece_hashes(),
        torrent_length,
        piece_length,
    )
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
            let mut handshake = Handshake::new(
                torrent.info.get_info_hash(),
                [0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9],
            );

            let _ = handshake.handshake(ip_data).await;

            println!("Peer ID: {}", handshake.response_peer_id.unwrap());
        }
        Command::DownloadPiece => {
            let path_to_save = &args[3];
            let torrent_file_name = &args[4];
            let piece_number = args[5].parse::<usize>().unwrap();

            let (mut stream, _, torrent_length, piece_length) =
                init_download(torrent_file_name).await;

            let piece =
                download_piece(&mut stream, torrent_length, piece_length, piece_number).await;

            save_pieces_to_file(path_to_save, piece.as_slice()).await;
        }
        Command::Download => {
            let path_to_save = &args[3];
            let torrent_file_name = &args[4];
            let (mut stream, hashes, torrent_length, piece_length) =
                init_download(torrent_file_name).await;

            let mut pieces: Vec<u8> = vec![];

            for (index, _) in hashes.iter().enumerate() {
                let piece = download_piece(&mut stream, torrent_length, piece_length, index).await;
                pieces.extend_from_slice(&piece);
            }

            save_pieces_to_file(path_to_save, pieces.as_slice()).await;
        }
        Command::Unknown => {
            println!("unknown command: {}", args[1])
        }
    }
}
