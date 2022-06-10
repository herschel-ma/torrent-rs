use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex,
    },
    thread::{self, JoinHandle},
};

use sha1::{Digest, Sha1};
use tokio::runtime::Handle;

use crate::{
    field::{
        constant::{COMPLETE, EMPTY},
        ByteField,
    },
    file::write_subpiece,
    tcp_bt::{connect::Connector, msg::structs::Piece},
    torrent::Client,
};

// struct for holding relevant values for hashing threads.
pub struct Hasher {
    pub queue: Mutex<VecDeque<Vec<Piece>>>,
    pub loops: Condvar,
    pub empty: Condvar,
    pub brk: AtomicBool,
}

impl Hasher {
    pub fn new() -> Hasher {
        Hasher {
            queue: Mutex::new(VecDeque::new()),
            loops: Condvar::new(),
            empty: Condvar::new(),
            brk: AtomicBool::new(false),
        }
    }
}

// spawns hashing threads
pub fn spawn_hash_write(
    hasher: &Arc<Hasher>,
    field: &Arc<Mutex<ByteField>>,
    client: &Arc<Client>,
    connecter: &Arc<Connector>,
    handle: Handle,
    threads: usize,
) -> Vec<JoinHandle<()>> {
    let mut handles = Vec::new();
    for i in 0..threads {
        let hasher = Arc::clone(hasher);
        let piece_field = Arc::clone(&field);
        let client = Arc::clone(client);
        let files = Arc::clone(&client.files);
        let connecter = Arc::clone(connecter);
        let handle = handle.clone();

        let builder = thread::Builder::new().name(format!("Hasher{}", i));
        let handle = builder
            .spawn(move || {
                loop {
                    let mut piece;
                    {
                        let mut guard = hasher
                            .loops
                            .wait_while(hasher.queue.lock().unwrap(), |q| {
                                if hasher.brk.load(Ordering::Relaxed) {
                                    return false;
                                }
                                q.is_empty()
                            })
                            .unwrap();
                        if hasher.brk.load(Ordering::Relaxed) {
                            break;
                        }
                        piece = match guard.pop_front() {
                            Some(t) => t,
                            None => break,
                        }
                    }
                    hasher.empty.notify_all();
                    let index = piece[0].index as usize;
                    let mut flat_piece = Vec::with_capacity(client.piece_len);
                    piece.sort_by_key(|x| x.begin);
                    for s in &piece {
                        flat_piece.extend_from_slice(&s.data); // assumes ordered by begin.
                    }

                    let mut hasher = Sha1::new();
                    hasher.update(flat_piece);
                    let piece_hash = hasher.finalize().to_vec();

                    if piece_hash
                        .iter()
                        .zip(&client.hashes[index])
                        .filter(|(a, b)| *a == *b)
                        .count()
                        != 20
                    {
                        {
                            // unreserve piece
                            let mut pf = piece_field.lock().unwrap();
                            pf.arr[index] = EMPTY;
                            // notify waiting connections
                            connecter.piece.notify_one();
                        }
                        continue;
                    }
                    for s in &piece {
                        handle.block_on(write_subpiece(s, client.piece_len, &files));
                    }
                    {
                        // critial section
                        let mut pf = piece_field.lock().unwrap();
                        pf.arr[index] = COMPLETE;
                    }
                }
            })
            .unwrap();
        handles.push(handle);
    }
    handles
}

// splits hashes from 1D rasterized to 2D.
pub fn split_hashes(hashes: &[u8]) -> Vec<Vec<u8>> {
    let num_pieces = hashes.len() / 20;
    let mut split_hashes: Vec<Vec<u8>> = vec![vec![0; 0]; num_pieces];
    for i in 0..num_pieces {
        split_hashes[i].extend_from_slice(&hashes[i * 20..((i + 1) * 20)]);
    }
    split_hashes
}
