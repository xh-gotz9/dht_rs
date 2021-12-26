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

    /// # Return
    /// if bucket don't need to splite self, return None. Else it will return
    /// a tuple of splite midlle NodeID and a new Bucket
    pub fn try_splite(&mut self) -> Option<(NodeID, Bucket)> {
        if self.size() > BUCKET_MAX_CAPACITY {
            Some(self.splite())
        } else {
            None
        }
    }

    fn splite(&mut self) -> (NodeID, Bucket) {
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

        (m, b)
    }
}

const BUCKET_MAX_CAPACITY: usize = 8;

struct Index {
    key: NodeID,
    left: KademliaNodeRef,
    right: KademliaNodeRef,
}

enum NodeValue {
    None,
    Index(Index),
    Bucket(Bucket),
}

type KademliaNodeRef = Rc<RefCell<KademliaNode>>;

struct KademliaNode {
    parent: Option<KademliaNodeRef>,
    value: NodeValue,
}

pub struct KademliaTable {
    root: KademliaNodeRef,
}

impl KademliaTable {
    pub fn new() -> KademliaTable {
        return KademliaTable {
            root: Rc::new(RefCell::new(KademliaNode {
                parent: None,
                value: NodeValue::Bucket(Bucket::new(MIN_HASH, MAX_HASH)),
            })),
        };
    }

    /// splite bucket when bucket is full
    pub fn insert_node(&mut self, node: Node) {
        let node_id = node.id;
        let n = self.find_bucket_index(node_id);

        let mut n_ref = n.as_ref().borrow_mut();

        if let NodeValue::Bucket(b) = &mut n_ref.value {
            b.insert(node);

            if let Some((k, b)) = b.try_splite() {
                // create new index node
                let mut right = KademliaNode {
                    parent: None,
                    value: NodeValue::Bucket(b),
                };

                let index_node = KademliaNode {
                    parent: n_ref.parent.as_ref().map(|p| Rc::clone(p)),
                    value: NodeValue::None,
                };

                let index_node_ref = Rc::new(RefCell::new(index_node));

                right.parent = Some(Rc::clone(&index_node_ref));

                let index_value = NodeValue::Index(Index {
                    key: k,
                    left: Rc::clone(&n),
                    right: Rc::new(RefCell::new(right)),
                });

                index_node_ref.as_ref().borrow_mut().value = index_value;

                // parent

                if let Some(p) = &n_ref.parent {
                    let p_rc = Rc::clone(p);
                    let mut p_ref = p_rc.as_ref().borrow_mut();

                    let v_ref = &mut p_ref.value;

                    if let NodeValue::Index(i) = v_ref {
                        if node_id > i.key {
                            i.left = Rc::clone(&index_node_ref);
                        } else {
                            i.right = Rc::clone(&index_node_ref);
                        }
                    } else {
                        panic!("kademlia table tree structure error")
                    }
                } else {
                    // set root
                    self.root = Rc::clone(&index_node_ref);
                }

                n_ref.parent = Some(index_node_ref);
            }
        }
    }

    fn find_bucket_index(&self, id: NodeID) -> KademliaNodeRef {
        let mut i = Rc::clone(&self.root);

        loop {
            let next = match &i.as_ref().borrow().value {
                NodeValue::Bucket(_) | &NodeValue::None => break,
                NodeValue::Index(index) => {
                    if id <= index.key {
                        Rc::clone(&index.left)
                    } else {
                        Rc::clone(&index.right)
                    }
                }
            };

            i = next;
        }

        i
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        hash::{mid, MAX_HASH, MIN_HASH},
        kademlia::NodeValue,
        node::Node,
    };

    use super::KademliaTable;

    #[test]
    fn auto_splite_test() {
        let mut table = KademliaTable::new();
        let mut v = vec![];
        for _i in 0..9 {
            let n = Node::random();
            v.push(n.id);

            table.insert_node(n);
        }

        let root = table.root.as_ref().borrow_mut();

        assert!(root.parent.is_none());
        assert!(matches!(root.value, NodeValue::Index(_)));

        if let NodeValue::Index(i) = &root.value {
            assert_eq!(i.key, mid(&MAX_HASH, &MIN_HASH));
        }

        // TODO: check bucket splited 
    }
}
