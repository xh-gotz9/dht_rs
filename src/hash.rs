use rand::prelude::*;
use std::fmt::{self, Debug, Result};

pub const HASH_LENGTH: usize = 20;

pub const MIN_HASH: Hash = Hash {
    val: [u8::MIN; HASH_LENGTH],
};

pub const MAX_HASH: Hash = Hash {
    val: [u8::MAX; HASH_LENGTH],
};

#[derive(Eq, PartialEq)]
pub struct Hash {
    pub val: [u8; HASH_LENGTH],
}

impl Clone for Hash {
    fn clone(&self) -> Self {
        Hash {
            val: self.val.clone(),
        }
    }
}

impl Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result {
        f.debug_struct("Hash")
            .field("val", &hex::encode_upper(&self.val))
            .finish()
    }
}

impl ToString for Hash {
    fn to_string(&self) -> std::string::String {
        hex::encode(self.val)
    }
}

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

    pub fn id_clone(&self) -> [u8; HASH_LENGTH] {
        self.val.clone()
    }

    pub fn raw_id(&self) -> Vec<u8> {
        self.val.to_vec()
    }
}
