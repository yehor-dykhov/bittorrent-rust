use regex::{Captures, Regex};
use serde_json::{Map, Number, Value};

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

pub fn decode_bencoded_value(encoded_value: &str) -> Value {
    match BencodedType::from(encoded_value) {
        BencodedType::List => Value::Array(list_parser(encoded_value).0),
        BencodedType::Dictionary => Value::Object(dictionary_parser(encoded_value).0),
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

fn number_parser(line: &str, start: usize) -> (Value, usize) {
    let num_sep: Regex = Regex::new(r"i(\d+)e").unwrap();
    let cap: Captures = num_sep.captures(&line[start..]).unwrap();
    let num_len: usize = cap[1].len();
    let num = cap[1].parse::<i64>().unwrap();

    (Value::Number(Number::from(num)), start + num_len + 1)
}

fn string_parser(line: &str, start: usize) -> (Value, usize) {
    let word_sep: Regex = Regex::new(r"(\d+):(.+)").unwrap();
    let cap: Captures = word_sep.captures(&line[start..]).unwrap();
    // println!("cap: {:?}", cap);
    let size_len = cap[1].len();
    let size = cap[1].parse::<usize>().unwrap();
    let word: &str = &cap[2][..size];
    // println!("word: {:?}", word);

    (
        Value::String(word.to_string()),
        start + size_len + word.len(),
    )
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

fn dictionary_parser(encoded_dictionary: &str) -> (Map<String, Value>, usize) {
    let num_regx: Regex = Regex::new(r"\d+").unwrap();

    let mut dictionary: Map<String, Value> = Map::new();
    let mut is_key: bool = true;
    let mut key: Option<String> = None;
    let len = encoded_dictionary.len();
    let mut start: usize = 1;

    while start < len {
        let ch = &encoded_dictionary[start..start + 1].chars().next().unwrap();
        let start_num = num_regx.is_match(&ch.to_string());
        let is_num = 'i' == *ch;
        let is_dictionary = 'd' == *ch;
        let is_list = 'l' == *ch;

        if start_num || is_num || is_dictionary || is_list {
            let mut end: usize = 0;
            let mut step_result: Option<Value> = None;

            if start_num {
                let (v, _end) = string_parser(encoded_dictionary, start);
                end = _end;
                step_result = Some(v);
            } else if is_num {
                let (v, _end) = number_parser(encoded_dictionary, start);
                end = _end;
                step_result = Some(v);
            } else if is_dictionary {
                let (_d, _end) = dictionary_parser(&encoded_dictionary[start..]);
                end = _end;
                step_result = Some(Value::from(_d));
            } else if is_list {
                let (_l, _end) = list_parser(&encoded_dictionary[start..]);
                end = _end;
                step_result = Some(Value::Array(_l));
            }

            if is_key {
                key = Some(
                    step_result
                        .take()
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .parse()
                        .unwrap(),
                );
            } else {
                dictionary.insert(key.take().unwrap(), step_result.take().unwrap());
            }

            is_key = !is_key;
            start = end + 1;

            continue;
        }

        break;
    }

    (dictionary, start + 1)
}
