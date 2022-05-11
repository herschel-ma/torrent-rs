use sha1::{Digest, Sha1};

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
