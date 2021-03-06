use std::{
    fmt::Display,
    io::Error,
    net::{SocketAddr, ToSocketAddrs},
    str::from_utf8,
};

use sha1::{Digest, Sha1};

use crate::bencode::Item;

use self::{http::http_announce, udp::udp_announce};
pub mod http;
pub mod udp;

// computes info_hash from .torrent bytes.
pub fn get_info_hash(mut bytes: Vec<u8>) -> [u8; 20] {
    let mut len: usize = 0;
    for c in bytes.windows(7) {
        len += 1;
        if c == b"4:infod" {
            break;
        }
    }
    bytes.drain(0..len + 5);
    bytes.pop();

    let mut hashser = Sha1::new();
    hashser.update(&bytes);
    hashser.finalize().into()
}
pub struct IpPort {
    pub ip: u32,
    pub port: u16,
}

impl IpPort {
    // takes in byte string of ip:port pairs and parses them
    fn from_bytes(bytes: &[u8]) -> Vec<Self> {
        let mut peers: Vec<IpPort> = vec![];
        if bytes.len() % 6 != 0 {
            return peers;
        }
        for chunk in bytes.chunks(6) {
            // IpPort is u32 ip, u16 port, 6 bytes.
            let peer: IpPort = IpPort {
                // big endian
                ip: u32::from_ne_bytes([chunk[3], chunk[2], chunk[1], chunk[0]]),
                port: u16::from_ne_bytes([chunk[5], chunk[4]]),
            };

            peers.push(peer)
        }

        peers
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Addr {
    Udp(SocketAddr),
    Http(SocketAddr),
}

impl Display for Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Addr::Udp(s) => write!(f, "Udp: {}", s),
            Addr::Http(s) => write!(f, "Http: {}", s),
        }
    }
}

pub fn make_addr(announce: &Item) -> Result<Addr, String> {
    let mut url = announce.get_string();
    // get url URI i.e udp://
    let mut count = 0;
    let mut len = 0;
    for c in &url {
        if count == 2 {
            break;
        }
        if *c == b'/' {
            count += 1
        }
        len += 1;
    }
    let mut addr;
    // handle each URI
    let udp = url[0] == b'u';
    match &url[0..len] {
        b"http://" => url.drain(0.."http://".len()),
        b"udp://" => url.drain(0.."udp://".len()),
        b"https://" => return Err("HTTPS/TLS not supported".to_string()),
        _ => return Err(format!("unknown URI: {}", from_utf8(&url).unwrap())),
    };
    // remove any /announce
    match url.iter().find(|i| **i == b'/') {
        None => {}
        Some(_) => loop {
            url.pop();
            if *url.last().unwrap() == b'/' {
                url.pop();
                break;
            }
        },
    }

    // add port number if none, default 80.
    addr = from_utf8(&url).unwrap().to_string();
    match url.last().unwrap() {
        b'0'..=b'9' => {}
        _ => addr.push_str(":80"),
    }

    // resole socket addr
    match addr.to_socket_addrs().unwrap().next() {
        Some(s) => {
            if udp {
                Ok(Addr::Udp(s))
            } else {
                Ok(Addr::Http(s))
            }
        }
        None => Err("no addr resolved".to_string()),
    }
}

// gets announce url
// 1. remove URI and /announce.
// 2. check if udp.
// 3. get default port if not seted.
// 4. check if addr was a valid socket address.
pub fn get_addr(tree: &Vec<Item>) -> Result<Addr, String> {
    let dict = tree[0].get_dict();
    match dict.get("announce".as_bytes()) {
        Some(s) => match make_addr(&s) {
            Ok(s) => Ok(s),
            Err(e) => match dict.get("announce-list".as_bytes()) {
                Some(l) => {
                    for i in l.get_list() {
                        if let Ok(s) = make_addr(&i.get_list()[0]) {
                            return Ok(s);
                        }
                    }
                    Err(e)
                }
                None => Err(e),
            },
        },
        None => Err("no announce url found".to_string()),
    }
}

pub async fn announce(addr: Addr, info_hash: [u8; 20], port: u16) -> Result<Vec<IpPort>, Error> {
    match addr {
        Addr::Http(a) => http_announce(a, info_hash, port).await,
        Addr::Udp(a) => udp_announce(a, info_hash, port).await,
    }
}
