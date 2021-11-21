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
        let data: [u8; HASH_LENGTH] = [0; HASH_LENGTH];
        return Hash { val: data };
    }

    pub fn wrap(val: [u8; HASH_LENGTH]) -> Hash {
        return Hash { val: val };
    }

    pub fn random() -> Hash {
        let mut data = [0; HASH_LENGTH];
        let mut rng = rand::thread_rng();
        for i in 0..HASH_LENGTH {
            let v: u8 = rng.gen();
            data[i] = v;
        }
        return Hash { val: data };
    }
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
