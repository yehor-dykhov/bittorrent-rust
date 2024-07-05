extern crate core;

use regex::{Captures, Regex};
use serde_json::{Number, Value};
use std::env;

enum BencodedType {
    List,
    Number,
    String,
    Unknown,
}

impl From<&str> for BencodedType {
    fn from(value: &str) -> Self {
        let first_char = value.chars().next().unwrap();
        match &first_char {
            'i' => Self::Number,
            'l' => Self::List,
            _ => {
                if let Ok(_s) = &first_char.to_string().parse::<usize>() {
                    return  Self::String
                }

                Self::Unknown
            },
        }
    }
}

fn number_parser(line: &str, start: usize) -> (Value, usize) {
    let num_sep: Regex = Regex::new(r"i(\d+)e").unwrap();
    let cap: Captures = num_sep.captures(&line[start..]).unwrap();
    let num_len: usize = cap[1].len();
    let num = cap[1].parse::<i64>().unwrap();

    (Value::Number(Number::from(num)), start + num_len + 1)
}

fn string_parser(line: &str, start: usize) -> (Value, usize) {
    let word_sep: Regex = Regex::new(r"(\d+):([a-zA-Z/:\-.]+)").unwrap();
    let cap: Captures = word_sep.captures(&line[start..]).unwrap();
    let size_len = cap[1].len();
    let size = cap[1].parse::<usize>().unwrap();
    let word: &str = &cap[2][..size];

    (Value::String(word.to_string()), start + size_len + word.len())
}

fn list_parser(encoded_list: &str) -> (Vec<Value>, usize) {
    let num_regx: Regex = Regex::new(r"\d+").unwrap();

    let mut list: Vec<Value> = vec![];
    let len = encoded_list.len();
    let mut start: usize = 1;

    while start < len {
        let ch = &encoded_list[start..start + 1];

        if num_regx.is_match(ch) {
            let (v, end) = string_parser(encoded_list, start);
            start = end + 1;
            list.push(v);
            continue;
        } else if let "i" = ch {
            let (v, end) = number_parser(encoded_list, start);
            start = end + 1;
            list.push(v);
            continue;
        } else if let "l" = ch {
            let (_l, _end) = list_parser(&encoded_list[start..]);
            start = _end + 1;

            list.push(Value::Array(_l));
            continue;
        }

        break;
    }

    (list, start + 1)
}

fn decode_bencoded_value(encoded_value: &str) -> Value {
    match BencodedType::from(encoded_value) {
        BencodedType::List => Value::Array(list_parser(encoded_value).0),
        BencodedType::Number => {
            let start = encoded_value.find('i').unwrap();
            let end = encoded_value.find('e').unwrap();
            let number_string = &encoded_value[(start + 1)..end];
            let number = number_string.parse::<i64>().unwrap();

            Value::Number(Number::from(number))
        }
        BencodedType::String => string_parser(encoded_value, 0).0,
        BencodedType::Unknown => {
            panic!("Unhandled encoded value: {}", encoded_value)
        }
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value);
    } else {
        println!("unknown command: {}", args[1])
    }
}
