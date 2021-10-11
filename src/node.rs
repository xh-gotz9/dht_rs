use std::{
    fmt::{self, Debug, Result},
    time::SystemTime,
};

pub type NodeID = crate::hash::Hash;

pub struct Node {
    pub id: NodeID,
    pub last_change: SystemTime,
}

const GOOD_NODE_DURATION_SECONDS: u64 = 60 * 15;

impl Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result {
        f.debug_struct("Node").field("id", &self.id).finish()
    }
}

impl Node {
    pub fn new(id: NodeID, last_change: SystemTime) -> Self {
        Self { id, last_change }
    }

    pub fn random() -> Node {
        return Node {
            id: NodeID::random(),
            last_change: SystemTime::now(),
        };
    }

    pub fn is_good(&self) -> bool {
        self.last_change
            .elapsed()
            .and_then(|x| Ok(x.as_secs() < GOOD_NODE_DURATION_SECONDS))
            .unwrap_or(false)
    }
}
