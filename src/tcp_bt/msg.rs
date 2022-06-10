// parse for tcp peer wire message
#![allow(dead_code)]

pub mod bytes {
    pub const CHOKE: u8 = 0;
    pub const UNCHOKE: u8 = 1;
    pub const INTERESTED: u8 = 2;
    pub const NOT_INTERESTED: u8 = 3;
    pub const HAVE: u8 = 4;
    pub const BITFIELD: u8 = 5;
    pub const REQUEST: u8 = 6;
    pub const PIECE: u8 = 7;
    pub const CANCEL: u8 = 8;
    pub const HANDSHAKE: u8 = 0x54;
}

// take off top 4 bytes to make a u32.
pub fn parse_u32(bytes: &[u8]) -> u32 {
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&bytes[0..4]);
    u32::from_be_bytes(buf)
}

pub mod structs {

    use serde::Serialize;

    use super::{
        bytes::{BITFIELD, CANCEL, HANDSHAKE, HAVE, PIECE, REQUEST},
        parse_u32,
    };
    #[derive(Debug, Serialize)]
    pub struct Handshake {
        pub pstrlen: u8,
        pub pstr: [u8; 19],
        pub reserved: [u8; 8],
        pub info_hash: [u8; 20],
        pub peer_id: [u8; 20],
    }

    impl Default for Handshake {
        fn default() -> Self {
            let name = "BitTorrent protocol";
            let mut p = [0u8; 19];
            p.copy_from_slice(&name.as_bytes());
            Handshake {
                pstrlen: 19,
                pstr: p,
                reserved: [0u8; 8],
                info_hash: [0u8; 20],
                peer_id: [0u8; 20],
            }
        }
    }
    impl Handshake {
        fn test(&self) -> bool {
            if self.pstrlen != 19 {
                return false;
            }
            self.pstr
                .iter()
                .zip(b"BitTorrent protocol")
                .all(|(a, b)| *a == *b)
        }

        pub fn parse(msg: &mut Vec<u8>) -> Option<Self> {
            if msg.len() < 68 {
                return None;
            }
            let mut handshake = Handshake {
                pstrlen: msg[0],
                ..Handshake::default()
            };
            handshake.pstr.copy_from_slice(&msg[1..20]);
            handshake.reserved.copy_from_slice(&msg[20..28]);
            handshake.info_hash.copy_from_slice(&msg[28..48]);
            handshake.peer_id.copy_from_slice(&msg[48..68]);
            if handshake.test() {
                msg.drain(0..68);
                Some(handshake)
            } else {
                None
            }
        }
    }

    #[derive(Debug, Serialize, Clone, Default)]
    // Message header without payload
    // https://wiki.theory.org/BitTorrentSpecification#Message_IDs
    pub struct Header {
        pub len: u32,
        pub id: u8,
    }

    impl Header {
        pub fn test(&self) -> bool {
            self.id <= CANCEL || self.id == HANDSHAKE
        }

        pub fn parse(msg: &[u8]) -> Option<Self> {
            if msg.len() < 5 {
                return None;
            }
            let head = Header {
                len: parse_u32(&msg[0..4]),
                id: msg[4],
            };

            if head.test() {
                Some(head)
            } else {
                None
            }
        }

        pub fn as_bytes(&self) -> Vec<u8> {
            let mut buf = Vec::new();
            buf.append(&mut u32::to_be_bytes(self.len).to_vec());
            buf.push(self.id);
            buf
        }
    }

    #[derive(Debug, Serialize, Default, Clone)]
    pub struct Have {
        // fixed length
        pub header: Header,
        pub index: u32,
    }

    impl Have {
        pub fn test(&self) -> bool {
            !(self.header.id != HAVE && self.header.len != 5)
        }

        pub fn parse(msg: &mut Vec<u8>) -> Option<Self> {
            if msg.len() < 9 {
                return None;
            }
            let have = Have {
                header: Header {
                    len: parse_u32(&msg[0..4]),
                    id: msg[4],
                },
                index: parse_u32(&msg[5..9]),
            };
            if have.test() {
                msg.drain(0..9);
                Some(have)
            } else {
                None
            }
        }
    }

    #[derive(Debug, Serialize, Default)]
    pub struct Bitfield {
        pub header: Header,
        pub data: Vec<u8>,
    }

    impl Bitfield {
        pub fn test(&self) -> bool {
            if self.header.id != BITFIELD {
                return false;
            }
            self.header.len as usize == self.data.len() + 1
        }
        pub fn parse(msg: &mut Vec<u8>) -> Option<Self> {
            let mut bitfield = Bitfield {
                header: Header::parse(&msg)?,
                ..Bitfield::default()
            };
            if msg.len() < (bitfield.header.len + 4) as usize {
                return None;
            }
            bitfield
                .data
                .extend_from_slice(&msg[5..(bitfield.header.len + 4) as usize]);
            if bitfield.test() {
                // msg.drain(0..(bitfield.header.len + 4) as usize);
                let mut copy = &mut msg[((bitfield.header.len + 4) as usize)..].to_vec();
                msg.clear();
                msg.append(&mut copy);
                Some(bitfield)
            } else {
                None
            }
        }
    }

    #[derive(Debug, Serialize, Default)]
    // https://wiki.theory.org/BitTorrentSpecification#Request
    pub struct Request {
        pub header: Header,
        pub index: u32,
        pub begin: u32,
        pub length: u32,
    }

    impl Request {
        pub fn test(&self) -> bool {
            if self.header.id != REQUEST {
                return false;
            }
            self.header.len == 13
        }

        pub fn parse(msg: &mut Vec<u8>) -> Option<Self> {
            if msg.len() < 17 {
                return None;
            }
            let req = Self {
                header: Header::parse(&msg[0..5])?,
                index: parse_u32(&msg[5..9]),
                begin: parse_u32(&msg[9..13]),
                length: parse_u32(&msg[13..17]),
            };

            if req.test() {
                msg.drain(0..17);
                Some(req)
            } else {
                None
            }
        }
    }

    #[derive(Debug, Default, Clone)]
    pub struct Piece {
        pub header: Header,
        pub index: u32,
        pub begin: u32,
        pub data: Vec<u8>,
    }

    impl Piece {
        pub fn test(&self) -> bool {
            if self.header.id != PIECE {
                return false;
            }
            self.header.len as usize == (self.data.len() + 9)
        }

        pub fn parse(msg: &mut Vec<u8>) -> Option<Self> {
            let mut piece = Piece {
                header: Header::parse(&msg)?,
                ..Piece::default()
            };

            if msg.len() < (piece.header.len + 4) as usize {
                return None;
            }

            piece.index = parse_u32(&msg[5..9]);
            piece.begin = parse_u32(&msg[9..13]);

            // !understand following lines

            unsafe {
                piece.data = Vec::new();
                piece.data.reserve((piece.header.len - 9) as usize);
                piece.data.set_len((piece.header.len - 9) as usize);
                let src = msg.as_ptr().add(13);
                let dst = piece.data.as_mut_ptr();
                std::ptr::copy_nonoverlapping(src, dst, (piece.header.len - 9) as usize);
                piece.data.set_len((piece.header.len - 9) as usize);
            }

            if piece.test() {
                let x = msg.len() - (piece.header.len + 4) as usize;
                unsafe {
                    std::ptr::drop_in_place(std::ptr::slice_from_raw_parts_mut(
                        msg.as_mut_ptr(),
                        (piece.header.len + 4) as usize,
                    ));
                    let src = msg.as_ptr().add((piece.header.len + 4) as usize);
                    let dst = msg.as_mut_ptr();
                    std::ptr::copy(src, dst, x);
                    msg.set_len(x);
                }
                Some(piece)
            } else {
                None
            }
        }

        pub fn as_bytes(&self) -> Vec<u8> {
            let mut bytes = Vec::new();
            bytes.append(&mut self.header.as_bytes());
            bytes.append(&mut u32::to_be_bytes(self.index).to_vec());
            bytes.append(&mut u32::to_be_bytes(self.begin).to_vec());
            bytes.extend_from_slice(&self.data);
            bytes
        }
    }

    #[derive(Debug, Default, Serialize)]
    // https://wiki.theory.org/BitTorrentSpecification#Cancel
    pub struct Cancel {
        pub header: Header,
        pub index: u32,
        pub begin: u32,
        pub length: u32,
    }

    impl Cancel {
        pub fn test(&self) -> bool {
            if self.header.id != CANCEL {
                return false;
            }
            self.header.len == 13
        }

        pub fn parse(msg: &mut Vec<u8>) -> Option<Self> {
            if msg.len() < 17 {
                return None;
            }
            let cancel = Cancel {
                header: Header::parse(&msg[0..5])?,
                index: parse_u32(&msg[5..9]),
                begin: parse_u32(&msg[9..13]),
                length: parse_u32(&msg[13..17]),
            };
            if cancel.test() {
                msg.drain(0..17);
                Some(cancel)
            } else {
                None
            }
        }
    }
}
use self::{bytes::*, structs::*};

pub const SUBPIECE_LEN: u32 = 0x4000; // 2^14 = 16384

// enum for each type message
pub enum Message {
    Handshake(Handshake),
    Choke(Header),
    Unchoke(Header),
    Interested(Header),
    NotInterested(Header),
    Have(Have),
    Bitfield(Bitfield),
    Request(Request),
    Piece(Piece),
    Cancel(Cancel),
}

fn is_zero(msg: &[u8]) -> bool {
    if msg.is_empty() {
        return false;
    }

    for i in msg {
        if *i != 0 {
            return false;
        }
    }
    true
}

// parse peer wire message
fn parse_msg(msg: &'static mut Vec<u8>) -> Vec<Message> {
    let mut messages: Vec<Message> = Vec::new();
    loop {
        if is_zero(msg) {
            break;
        }
        let _byte = match msg.get(0) {
            Some(byte) => *byte,
            None => break,
        };

        let byte = match msg.get(4) {
            Some(byte) => *byte,
            None => break,
        };

        match byte {
            CHOKE => messages.push(Message::Choke(Header::parse(msg).unwrap())),
            UNCHOKE => messages.push(Message::Unchoke(Header::parse(msg).unwrap())),
            INTERESTED => messages.push(Message::Interested(Header::parse(msg).unwrap())),
            NOT_INTERESTED => messages.push(Message::NotInterested(Header::parse(msg).unwrap())),
            HAVE => messages.push(Message::Have(Have::parse(msg).unwrap())),
            BITFIELD => messages.push(Message::Bitfield(Bitfield::parse(msg).unwrap())),
            REQUEST => messages.push(Message::Request(Request::parse(msg).unwrap())),
            PIECE => messages.push(Message::Piece(Piece::parse(msg).unwrap())),
            CANCEL => messages.push(Message::Cancel(Cancel::parse(msg).unwrap())),
            _ => {
                // println!("{:?}", msg);
                unreachable!("parse message");
            }
        }
    }
    messages
}

// returns whether the current message buffer is parseable or not
fn try_parse(orignal: &[u8]) -> bool {
    if orignal.is_empty() {
        return false;
    }
    let mut msg = orignal.to_vec();
    loop {
        if is_zero(&msg) {
            return false;
        }

        let _byte = match msg.get(0) {
            Some(byte) => *byte,
            None => return false,
        };

        let byte = match msg.get(4) {
            Some(byte) => *byte,
            None => return false,
        };

        match byte {
            CHOKE | UNCHOKE | INTERESTED | NOT_INTERESTED => {
                if Header::parse(&msg).is_none() {
                    return false;
                }
            }
            HAVE => {
                if Have::parse(&mut msg).is_none() {
                    return false;
                }
            }
            BITFIELD => {
                if Bitfield::parse(&mut msg).is_none() {
                    return false;
                }
            }
            REQUEST => {
                if Request::parse(&mut msg).is_none() {
                    return false;
                }
            }
            PIECE => {
                if Piece::parse(&mut msg).is_none() {
                    return false;
                }
            }
            CANCEL => {
                if Cancel::parse(&mut msg).is_none() {
                    return false;
                }
            }
            HANDSHAKE => {
                if Handshake::parse(&mut msg).is_none() {
                    return false;
                }
            }
            _ => {
                // println!("{:?}", msg);
                return false;
            }
        }
    }
}

pub fn partial_parse(msg: &mut Vec<u8>) -> (bool, Vec<Message>) {
    let mut list: Vec<Message> = vec![];
    if msg.is_empty() {
        return (false, list);
    }
    loop {
        if is_zero(&msg) {
            return (false, list);
        }

        let _byte = match msg.get(0) {
            Some(byte) => *byte,
            None => return (true, list),
        };

        let byte = match msg.get(4) {
            Some(byte) => *byte,
            None => return (false, list),
        };

        match byte {
            CHOKE => match Header::parse(&msg) {
                Some(x) => {
                    msg.drain(0..5);
                    list.push(Message::Choke(x))
                }
                None => return (false, list),
            },
            UNCHOKE => match Header::parse(&msg) {
                Some(x) => {
                    msg.drain(0..5);
                    list.push(Message::Unchoke(x))
                }
                None => return (false, list),
            },
            INTERESTED => match Header::parse(&msg) {
                Some(x) => {
                    msg.drain(0..5);
                    list.push(Message::Interested(x))
                }
                None => return (false, list),
            },
            NOT_INTERESTED => match Header::parse(&msg) {
                Some(x) => {
                    msg.drain(0..5);
                    list.push(Message::NotInterested(x))
                }
                None => return (false, list),
            },
            HAVE => match Have::parse(msg) {
                Some(x) => list.push(Message::Have(x)),
                None => return (false, list),
            },
            BITFIELD => match Bitfield::parse(msg) {
                Some(x) => list.push(Message::Bitfield(x)),
                None => return (false, list),
            },
            REQUEST => match Request::parse(msg) {
                Some(x) => list.push(Message::Request(x)),
                None => return (false, list),
            },
            PIECE => match Piece::parse(msg) {
                Some(x) => list.push(Message::Piece(x)),
                None => return (false, list),
            },
            CANCEL => match Cancel::parse(msg) {
                Some(x) => list.push(Message::Cancel(x)),
                None => return (false, list),
            },
            HANDSHAKE => match Handshake::parse(msg) {
                Some(x) => list.push(Message::Handshake(x)),
                None => return (false, list),
            },
            _ => {
                // println!("{:?}", msg);
                return (false, list);
            }
        }
    }
}
