use std::{cell::RefCell, collections::BTreeMap};

use crate::{
    hash::Hash,
    node::{Node, NodeID},
};

type NodeContainer = BTreeMap<Hash, Node>;

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
}
