use super::Item;
use std::collections::BTreeMap;

fn parse_int(str: &mut Vec<u8>) -> usize {
    let mut len: usize = 0;
    let mut int_string: String = String::new();
    for c in str.iter() {
        len += 1;
        if *c == b'i' {
            continue;
        }
        if *c == b'e' {
            break;
        }
        int_string.push(*c as char);
    }
    str.drain(0..len);
    int_string.parse::<usize>().unwrap()
}

fn parse_string(str: &mut Vec<u8>) -> Vec<u8> {
    let mut int_len: usize = 0;
    let mut int_string: String = String::new();
    for s in str.iter() {
        int_len += 1;
        if *s == b':' {
            break;
        }
        int_string.push(*s as char);
    }
    str.drain(0..int_len);

    let len = int_string.parse::<usize>().unwrap();
    let string = str[..len].to_vec();
    let mut clone = str[len..].to_vec();
    str.clear();
    str.append(&mut clone);
    string
}

fn parse_list(str: &mut Vec<u8>) -> Vec<Item> {
    str.drain(..1);
    let mut list = Vec::<Item>::new();
    loop {
        match *str.get(0).unwrap() as char {
            'i' => list.push(Item::Integer(parse_int(str))),
            'l' => list.push(Item::List(parse_list(str))),
            'd' => list.push(Item::Dict(parse_dict(str))),
            '0'..='9' => list.push(Item::String(parse_string(str))),
            'e' => break,
            _ => unreachable!(),
        }
    }
    str.drain(..1);
    list
}

fn parse_dict(str: &mut Vec<u8>) -> BTreeMap<Vec<u8>, Item> {
    str.drain(0..1);
    let mut dict: BTreeMap<Vec<u8>, Item> = BTreeMap::new();
    loop {
        if *str.get(0).unwrap() == b'e' {
            break;
        }
        let s = parse_string(str);
        match *str.get(0).unwrap() as char {
            'i' => dict.insert(s, Item::Integer(parse_int(str))),
            'l' => dict.insert(s, Item::List(parse_list(str))),
            'd' => dict.insert(s, Item::Dict(parse_dict(str))),
            '0'..='9' => dict.insert(s, Item::String(parse_string(str))),
            _ => unreachable!(),
        };
    }
    str.drain(0..1);
    dict
}

pub fn parse(str: &mut Vec<u8>) -> Vec<Item> {
    // parse readed content from torrent file to Items, and push Item to Vec<Item>
    let mut items: Vec<Item> = Vec::new();
    while let Some(c) = str.get(0) {
        match *c {
            b'i' => items.push(Item::Integer(parse_int(str))),
            b'l' => items.push(Item::List(parse_list(str))),
            b'd' => items.push(Item::Dict(parse_dict(str))),
            b'0'..=b'9' => items.push(Item::String(parse_string(str))),
            _ => break,
        }
    }
    items
}
