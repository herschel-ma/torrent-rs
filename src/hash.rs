// splits hashes from 1D rasterized to 2D.
pub fn split_hashes(hashes: &[u8]) -> Vec<Vec<u8>> {
    let num_pieces = hashes.len() / 20;
    let mut split_hashes: Vec<Vec<u8>> = vec![vec![0; 0]; num_pieces];
    for i in 0..num_pieces {
        split_hashes[i].extend_from_slice(&hashes[i * 20..((i + 1) * 20)]);
    }
    split_hashes
}
