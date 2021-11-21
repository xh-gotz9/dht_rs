use std::{cell::RefCell, rc::Rc};

use crate::{
    bucket::Bucket,
    hash::{MAX_HASH, MIN_HASH},
    node::{Node, NodeID},
};

type BucketRef = Rc<RefCell<Bucket>>;

pub struct KademliaTable {
    buckets: Vec<BucketRef>,
}

impl KademliaTable {
    pub fn new() -> KademliaTable {
        return KademliaTable {
            buckets: vec![Rc::new(RefCell::new(Bucket::new(MIN_HASH, MAX_HASH)))],
        };
    }

    /// # TODO
    /// splite when bucket is full
    pub fn insert_node(&self, node: Node) {
        if let Some(b) = self.find_bucket_to_insert(node.id) {
            b.as_ref().borrow_mut().insert(node);
        } else {
            panic!("not found bucket of {:?}", node.id);
        }
    }

    fn find_bucket_to_insert(&self, id: NodeID) -> Option<BucketRef> {
        for ele in &self.buckets {
            let b = Rc::clone(&ele);

            if b.as_ref().borrow().node_in_range(id) {
                return Some(b);
            }
        }

        None
    }
}
