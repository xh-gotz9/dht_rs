use std::{
    fmt::{self, Debug, Result},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::SystemTime,
};

use chrono::{DateTime, Local};
use rand::Rng;

pub type NodeID = crate::hash::Hash;

pub struct Node {
    pub id: NodeID,
    pub addr: SocketAddr,
    pub last_active: SystemTime,
}

const GOOD_NODE_DURATION_SECONDS: u64 = 60 * 15;

impl Node {
    pub fn new(id: NodeID, addr: SocketAddr, last_active: SystemTime) -> Self {
        Self {
            id,
            addr,
            last_active,
        }
    }

    pub fn random() -> Node {
        let mut rd = rand::thread_rng();

        Self {
            id: NodeID::random(),
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), rd.gen()),
            last_active: SystemTime::now(),
        }
    }

    pub fn is_good(&self) -> bool {
        self.last_active
            .elapsed()
            .and_then(|x| Ok(x.as_secs() < GOOD_NODE_DURATION_SECONDS))
            .unwrap_or(false)
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result {
        let last_active = *(&self.last_active);
        let utc: DateTime<Local> = last_active.into();
        f.debug_struct("Node")
            .field("id", &self.id)
            .field("last_active", &utc.format("%Y-%m-%d %T").to_string())
            .finish()
    }
}
