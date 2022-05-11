use crate::{
    bencode::{decode::parse, Item},
    file::{parse_file, FileSize},
    hash::split_hashes,
    tracker::get_info_hash,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Client {
    pub tree: Vec<Item>,
    pub info_hash: [u8; 20],
    pub piece_len: usize,
    pub num_pieces: usize,
    pub hashes: Vec<Vec<u8>>,
    pub files: Arc<Vec<FileSize>>,
    pub file_len: usize,
}

impl Client {
    pub async fn new(bytes: &[u8]) -> Self {
        let mut copy = bytes.to_vec();
        let tree = parse(&mut copy);
        let dict = tree[0].get_dict();
        let info = dict.get("info".as_bytes()).unwrap().get_dict();
        let piece_len = info.get("piece length".as_bytes()).unwrap().get_integer();
        let num_pieces = info.get("pieces".as_bytes()).unwrap().get_string().len() / 20;
        let hashes = info.get("pieces".as_bytes()).unwrap().get_string();
        let split_hashes = split_hashes(&hashes);
        let (files, file_len) = parse_file(&info).await;

        Self {
            tree,
            // 20bits of sha1 hash->info value from meta file.
            info_hash: get_info_hash(bytes.to_vec()),
            piece_len,
            num_pieces,
            hashes: split_hashes,
            files,
            file_len,
        }
    }
}
