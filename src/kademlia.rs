use std::{cell::RefCell, rc::Rc};

use crate::{
    bucket::Bucket,
    hash::{MAX_HASH, MIN_HASH},
    node::{Node, NodeID},
};

const BUCKET_MAX_CAPACITY: usize = 8;

type BucketRef = Rc<RefCell<Bucket>>;

pub struct KademliaTable {
    buckets: RefCell<Vec<BucketRef>>,
}

impl KademliaTable {
    pub fn new() -> KademliaTable {
        return KademliaTable {
            buckets: RefCell::new(vec![Rc::new(RefCell::new(Bucket::new(MIN_HASH, MAX_HASH)))]),
        };
    }

    /// # TODO
    /// splite when bucket is full
    pub fn insert_node(&self, node: Node) {
        if let Some((i, b)) = self.find_bucket_to_insert(node.id) {
            let mut target = b.as_ref().borrow_mut();

            target.insert(node);

            if target.size() > BUCKET_MAX_CAPACITY {
                let b = target.splite();

                self.buckets
                    .borrow_mut()
                    .insert(i + 1, Rc::new(RefCell::new(b)));
            }
        } else {
            panic!("not found bucket of {:?}", node.id);
        }
    }

    fn find_bucket_to_insert(&self, id: NodeID) -> Option<(usize, BucketRef)> {
        for (i, ele) in self.buckets.borrow().iter().enumerate() {
            let b = Rc::clone(&ele);

            if b.as_ref().borrow().node_in_range(id) {
                return Some((i, b));
            }
        }

        None
    }
}
