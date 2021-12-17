use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::{
    hash::{self, Hash},
    hash::{MAX_HASH, MIN_HASH},
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

    pub fn node_in_range(&self, id: &NodeID) -> bool {
        return id >= &self.range_from && id < &self.range_to;
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

const BUCKET_MAX_CAPACITY: usize = 8;

type BucketRef = Rc<RefCell<Bucket>>;

pub struct KademliaTable {
    header: RefCell<Vec<BucketRef>>,
}

impl KademliaTable {
    pub fn new() -> KademliaTable {
        return KademliaTable {
            header: RefCell::new(vec![Rc::new(RefCell::new(Bucket::new(MIN_HASH, MAX_HASH)))]),
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

                self.header
                    .borrow_mut()
                    .insert(i + 1, Rc::new(RefCell::new(b)));
            }
        } else {
            panic!("not found bucket of {:?}", node.id);
        }
    }

    fn find_bucket_to_insert(&self, id: NodeID) -> Option<(usize, BucketRef)> {
        for (i, ele) in self.header.borrow().iter().enumerate() {
            let b = Rc::clone(&ele);

            if b.as_ref().borrow().node_in_range(&id) {
                return Some((i, b));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        hash::{self, MAX_HASH, MIN_HASH},
        node::Node,
    };
    use std::rc::Rc;

    use super::KademliaTable;

    #[test]
    fn auto_splite_test() {
        let table = KademliaTable::new();
        let mut v = vec![];
        for _i in 0..9 {
            let n = Node::random();
            v.push(n.id);

            table.insert_node(n);
        }

        // check bucket splited
        let b = Rc::clone(table.header.borrow().first().expect("buckets error"));
        let m = hash::mid(&MIN_HASH, &MAX_HASH);
        assert_eq!(b.as_ref().borrow().range_to, m);

        // check splite correctly
        table.header.borrow().iter().for_each(|b| {
            let b = Rc::clone(b);

            let b_ref = b.as_ref().borrow();
            let nodes = &b_ref.nodes;

            assert!(nodes.borrow().keys().all(|e| b_ref.node_in_range(e)));

            println!(
                "range from {:?} to {:?}: {:?}",
                b_ref.range_from,
                b_ref.range_to,
                nodes.borrow()
            )
        });
    }
}
