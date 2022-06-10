// byte field for torrent control flow.
#![allow(dead_code)]

pub mod constant {
    pub const EMPTY: u8 = 0;
    pub const IN_PROGRESS: u8 = 1;
    pub const COMPLETE: u8 = 2;
}

pub struct ByteField {
    pub arr: Vec<u8>,
}

impl ByteField {
    // returns true if every index marked COMPLETE.
    pub fn if_full(&self) -> bool {
        self.arr.iter().filter(|x| **x < constant::COMPLETE).count() == 0
    }

    // returns an index which markd EMPTY.
    pub fn get_empty(&self) -> Option<usize> {
        self.arr.iter().position(|x| *x == constant::EMPTY)
    }
}
