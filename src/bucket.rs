use std::{cell::RefCell, collections::BTreeMap};

use crate::{
    hash::{self, Hash},
    node::{Node, NodeID},
};

type NodeContainer = BTreeMap<Hash, Node>;

/// hold nodes which id value in range of [range_from, range_to)
pub struct Bucket {
    range_from: Hash,
    range_to: Hash,
    nodes: RefCell<NodeContainer>,
}

impl Bucket {
    pub fn new(range_from: Hash, range_to: Hash) -> Bucket {
        Bucket {
            range_from,
            range_to,
            nodes: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn size(&self) -> usize {
        self.nodes.borrow().len()
    }

    pub fn node_in_range(&self, id: NodeID) -> bool {
        return id >= self.range_from && id < self.range_to;
    }

    /// insert node into bucket
    /// # Return
    /// if bucket is full, bucket will splite self and return new bucket
    // pub fn insert(&mut self, node: Node) -> Option<Bucket> {
    pub fn insert(&self, node: Node) {
        self.nodes.borrow_mut().insert(node.id, node);
    }

    pub fn splite(&mut self) -> Bucket {
        let m = hash::mid(&self.range_from, &self.range_to);

        let b = Bucket::new(m, self.range_to);

        // unstable feature
        // v.drain_filter(|k, v| k > &m);

        let nodes = std::mem::replace(&mut self.nodes, RefCell::new(BTreeMap::new())).into_inner();

        for (key, val) in nodes.into_iter() {
            if key >= m {
                b.insert(val);
            } else {
                self.insert(val);
            }
        }

        self.range_to = m;

        b
    }
}
