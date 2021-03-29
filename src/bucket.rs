use crate::node::{self, Node, NodeID};
use std::{
    fmt::{self, Result},
    sync::{Arc, Mutex},
};

/// 一个 Bucket 中仅可容纳 8 个 Node
const BUCKET_MAX_SIZE: usize = 8;

pub struct Bucket {
    id: NodeID,
    next: Option<Arc<Mutex<Bucket>>>,
    nodes: Option<Vec<Arc<Node>>>,
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
    pub fn new(id: NodeID, next: Option<Arc<Mutex<Bucket>>>) -> Bucket {
        Bucket {
            id: id,
            next: next,
            nodes: None,
        }
    }

    /// bucket 链表从大到小 (2^21-1) -> 0
    /// 较大 ID 值的 bucket 分割时放在链表前部, 便于搜索
    fn split_self(&mut self) {
        let i = node::id::lowest_bit(&self.id)
            .and_then(|x| Some(x + 1))
            .unwrap_or(1);
        let j = self
            .next
            .as_ref()
            .and_then(|x| {
                node::id::lowest_bit(&x.as_ref().lock().expect("multithread lock error").id)
            })
            .and_then(|x| Some(x + 1))
            .unwrap_or(1);
        let pos = usize::max(i, j);

        let arr = self.id.id_clone();
        self.id.val[pos / 8] = self.id.val[pos / 8] | (1 << (8 - pos % 8));

        // 新 bucket 放于最后
        let mut new_bucket = Bucket::new(
            NodeID::wrap(arr),
            self.next.as_ref().and_then(|x| Some(Arc::clone(x))),
        );

        let self_nodes = self.nodes.as_mut().expect("转移 node");
        let mut i = 0;
        while i != self_nodes.len() {
            if node::id::cmp(&self_nodes[i].id, &new_bucket.id) < 0 {
                let val = self_nodes.remove(i);
                new_bucket._insert_node(val);
            } else {
                i += 1;
            }
        }

        self.next = Some(Arc::new(Mutex::new(new_bucket)));
    }

    #[allow(unused)]
    pub fn insert_node(&mut self, node: Node) {
        // TODO 遍历查找 bucket, 移除 _insert_node 中的递归调用
        self._insert_node(Arc::new(node))
    }

    fn _insert_node(&mut self, node: Arc<Node>) {
        if let Some(v) = &self.next {
            let mut b = v.lock().expect("multithread lock error");
            if node::id::cmp(&node.id, &b.id) <= 0 {
                b._insert_node(node);
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

                    if node::id::cmp(&node.id, &self.id) >= 0 {
                        self._insert_node(node);
                    } else {
                        self.next
                            .as_ref()
                            .expect("splite failed")
                            .lock()
                            .expect("multithread lock error")
                            ._insert_node(node);
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

    pub fn node_id(&self) -> &NodeID {
        &self.id
    }

    pub fn next_bucket(&self) -> Option<Arc<Mutex<Bucket>>> {
        self.next.as_ref().and_then(|rc| Some(Arc::clone(rc)))
    }

    pub fn find_node(&self, id: &NodeID) -> Option<Arc<Node>> {
        if let Some(v) = &self.nodes {
            for node in v {
                if node::id::cmp(id, &node.id) == 0 {
                    return Some(Arc::clone(node));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use crate::bucket::Bucket;
    use crate::bucket::Rc;
    use crate::node::id::{self, NODE_ID_LENGTH};
    use crate::node::{Node, NodeID};
    use core::cell::RefCell;
    use std::{
        net::{SocketAddr, SocketAddrV4},
        time::SystemTime,
    };

    #[test]
    fn init_test() {
        let mut bucket = Bucket::new(NodeID::wrap([0; NODE_ID_LENGTH]), None);

        for _i in 0..9 {
            let node = Node::new(
                NodeID::rand(),
                SocketAddr::V4(SocketAddrV4::new(
                    "127.0.0.1".parse().expect("ip parse error"),
                    8080,
                )),
                SystemTime::now(),
            );
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
                                            Some(id::cmp(&n.id, &nb.as_ref().borrow().id) > 0)
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
