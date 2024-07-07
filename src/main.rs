mod parsers;

extern crate core;

use serde_json::{Number, Value};
use std::env;

enum BencodedType {
    List,
    Dictionary,
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
            'd' => Self::Dictionary,
            _ => {
                if let Ok(_s) = &first_char.to_string().parse::<usize>() {
                    return Self::String;
                }

                Self::Unknown
            }
        }
    }
}

fn decode_bencoded_value(encoded_value: &str) -> Value {
    match BencodedType::from(encoded_value) {
        BencodedType::List => Value::Array(parsers::list_parser(encoded_value).0),
        BencodedType::Dictionary => Value::Object(parsers::dictionary_parser(encoded_value).0),
        BencodedType::Number => {
            let start = encoded_value.find('i').unwrap();
            let end = encoded_value.find('e').unwrap();
            let number_string = &encoded_value[(start + 1)..end];
            let number = number_string.parse::<i64>().unwrap();

            Value::Number(Number::from(number))
        }
        BencodedType::String => parsers::string_parser(encoded_value, 0).0,
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
