use rand::prelude::*;
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Result},
};

pub const HASH_LENGTH: usize = 20;

#[derive(Clone, Copy, Eq, PartialEq, PartialOrd)]
pub struct Hash {
    val: [u8; HASH_LENGTH],
}

pub const MIN_HASH: Hash = Hash {
    val: [u8::MIN; HASH_LENGTH],
};

pub const MAX_HASH: Hash = Hash {
    val: [u8::MAX; HASH_LENGTH],
};

impl Hash {
    pub fn new() -> Hash {
        let val: [u8; HASH_LENGTH] = [0; HASH_LENGTH];

        Hash { val }
    }

    pub fn wrap(val: [u8; HASH_LENGTH]) -> Hash {
        Hash { val }
    }

    pub fn random() -> Hash {
        let mut val = [0; HASH_LENGTH];
        let mut rng = rand::thread_rng();
        for i in 0..val.len() {
            let v: u8 = rng.gen();
            val[i] = v;
        }

        Hash { val }
    }
}

pub fn mid(id1: &Hash, id2: &Hash) -> Hash {
    let mut node = Hash::new();
    let mut b: u16 = 0;
    for i in (0..HASH_LENGTH).rev() {
        let mid = id1.val[i] as u16 + id2.val[i] as u16 + b;
        node.val[i] = mid as u8;
        b = mid >> 8;
    }
    for i in 0..HASH_LENGTH {
        let tmp = node.val[i] as u16 + (b << 8);
        node.val[i] = (tmp / 2) as u8;
        b = tmp & 1;
    }
    return node;
}

impl ToString for Hash {
    fn to_string(&self) -> std::string::String {
        hex::encode(self.val)
    }
}

impl Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result {
        f.debug_struct("Hash")
            .field("val", &hex::encode_upper(&self.val))
            .finish()
    }
}

impl Ord for Hash {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let mut count = 0;
        while count < 20 && self.val[count] == other.val[count] {
            count += 1;
        }

        if count >= 20 {
            return Ordering::Equal;
        } else {
            return if self.val[count] > other.val[count] {
                Ordering::Greater
            } else {
                Ordering::Less
            };
        }
    }
}
