use crate::node::{self, Node, NodeID};
use std::cell::RefCell;
use std::fmt::{self, Result};
use std::rc::Rc;

/// 一个 Bucket 中仅可容纳 8 个 Node
const BUCKET_MAX_SIZE: usize = 8;

#[derive(Eq, PartialEq)]
pub struct Bucket {
    id: NodeID,
    next: Option<Rc<RefCell<Bucket>>>,
    nodes: Option<Vec<Node>>,
}

impl std::fmt::Debug for Bucket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result {
        f.debug_struct("Bucket")
            .field("id", &self.id)
            .field("nodes", &self.nodes)
            .finish()
    }
}

impl Bucket {
    #[allow(unused)]
    pub fn new(id: NodeID, next: Option<Rc<RefCell<Bucket>>>) -> Bucket {
        Bucket {
            id: id,
            next: next,
            nodes: None,
        }
    }

    #[allow(unused)]
    fn split_self(&mut self) {
        let i = node::id::lowest_bit(&self.id)
            .and_then(|x| Some(x + 1))
            .unwrap_or(1);
        let j = self
            .next
            .as_ref()
            .and_then(|x| node::id::lowest_bit(&x.borrow().id).and_then(|x| Some(x + 1)))
            .unwrap_or(1);
        let pos = usize::max(i, j);

        let mut arr = self.id.id_clone();
        arr[pos / 8] = arr[pos / 8] | (1 << (8 - pos % 8));
        let mut bucket = Bucket::new(
            NodeID::wrap(arr),
            self.next.as_ref().and_then(|x| Some(Rc::clone(x))),
        );

        let mut self_nodes = self.nodes.as_mut().expect("转移 node");
        let mut i = 0;
        while i != self_nodes.len() {
            if node::id::cmp(&self_nodes[i].id, &bucket.id) >= 0 {
                let val = self_nodes.remove(i);
                bucket.insert_node(val);
            } else {
                i += 1;
            }
        }

        self.next = Some(Rc::new(RefCell::new(bucket)));
    }

    #[allow(unused)]
    pub fn insert_node(&mut self, node: Node) {
        if let Some(v) = &self.next {
            let nb = Rc::clone(v);
            if node::id::cmp(&node.id, &nb.borrow().id) >= 0 {
                nb.borrow_mut().insert_node(node);
            }
            return;
        }

        match &mut self.nodes {
            Some(v) => {
                // 清理过期 node
                // TODO 不应该直接清理, 还要做 ping 检查
                v.retain(|x| x.is_good_node());

                if v.len() + 1 > BUCKET_MAX_SIZE {
                    self.split_self();

                    if let Some(next) = &self.next {
                        let b = Rc::clone(next);
                        if node::id::cmp(&node.id, &b.borrow().id) < 0 {
                            b.borrow_mut().insert_node(node);
                        }
                    } else {
                        self.insert_node(node);
                    }
                } else {
                    v.push(node);
                }
            }
            None => {
                self.nodes = Some(vec![node]);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::bucket::Bucket;
    use crate::bucket::Rc;
    use crate::node::id::{self, NODE_ID_LENGTH};
    use crate::node::{Node, NodeID};
    use core::cell::RefCell;
    use std::time::SystemTime;

    #[test]
    fn init_test() {
        let mut bucket = Bucket::new(NodeID::wrap([0; NODE_ID_LENGTH]), None);

        for _i in 0..9 {
            let node = Node::new(NodeID::rand(), SystemTime::now());
            bucket.insert_node(node);
        }

        assert_ne!(bucket.next, None);

        let bucket_ref = Rc::new(RefCell::new(bucket));
        let mut current_bucket = bucket_ref;

        loop {
            assert_eq!(
                Rc::clone(&current_bucket)
                    .borrow()
                    .nodes
                    .as_ref()
                    .and_then(|v| {
                        return Some(
                            v.len() == 0
                                || v.iter().all(|n| {
                                    current_bucket
                                        .borrow()
                                        .next
                                        .as_ref()
                                        .and_then(|nb| {
                                            Some(id::cmp(&n.id, &nb.as_ref().borrow().id) < 0)
                                        })
                                        .unwrap_or(true) // if current is last bucket,
                                }),
                        );
                    })
                    .unwrap_or(true), // if `nodes` is None, pass
                true
            );

            match Rc::clone(&current_bucket)
                .borrow()
                .next
                .as_ref()
                .and_then(|b| Some(Rc::clone(&b)))
            {
                Some(b) => current_bucket = b,
                None => break,
            }
        }
    }
}
