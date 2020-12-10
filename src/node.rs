use core::cell::Cell;
use id::{kademila::lowest_bit, NodeID};
use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

struct Node {
    id: NodeID,
}

/// 一个 Bucket 中仅可容纳 8 个 Node
const BUCKET_MAX_SIZE: usize = 8;

struct Bucket {
    id: NodeID,
    next: Option<Rc<RefCell<Bucket>>>,
    nodes: Option<Vec<Node>>,
}

impl Bucket {
    pub fn new(id: NodeID, next: Option<Rc<RefCell<Bucket>>>) -> Bucket {
        Bucket {
            id: id,
            next: next,
            nodes: None,
        }
    }

    fn split_self(&mut self) {
        let i = lowest_bit(&self.id).and_then(|x| Some(x + 1)).unwrap_or(0);
        let j = self
            .next
            .as_ref()
            .and_then(|x| lowest_bit(&x.as_ref().borrow().id).and_then(|x| Some(x + 1)))
            .unwrap_or(0);
        let pos = usize::max(i, j);

        let mut arr = self.id.id_clone();
        arr[pos / 8] = arr[pos / 8] & (1 << (pos % 8));

        let node = NodeID::wrap(arr);
        let bucket = Bucket::new(node, self.next.clone());

        // TODO 转移 Node

        self.next = Some(Rc::new(RefCell::new(bucket)));
    }

    pub fn insert_node(&mut self, node: Node) {
        match &mut self.nodes {
            Some(v) => {
                if v.len() + 1 > BUCKET_MAX_SIZE {
                    self.split_self();
                    let b = Rc::clone(&self.next.as_ref().unwrap());
                    if id::kademila::cmp(&node.id, &b.borrow_mut().id) < 0 {
                        b.borrow_mut().insert_node(node);
                    } else {
                        self.insert_node(node);
                    };
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

pub mod id {
    use rand::prelude::*;
    use std::fmt::{self, Debug, Result};

    const NODE_ID_LENGTH: usize = 20;

    pub struct NodeID {
        val: [u8; 20],
    }

    impl Debug for NodeID {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result {
            f.debug_struct("NodeID")
                .field("val", &hex::encode_upper(&self.val))
                .finish()
        }
    }

    impl NodeID {
        #[allow(unused)]
        pub fn new() -> NodeID {
            let data: [u8; NODE_ID_LENGTH] = [0; NODE_ID_LENGTH];
            return NodeID { val: data };
        }
        #[allow(unused)]
        pub fn wrap(val: [u8; 20]) -> NodeID {
            return NodeID { val: val };
        }
        #[allow(unused)]
        pub fn rand() -> NodeID {
            let mut data = [0; NODE_ID_LENGTH];
            let mut rng = rand::thread_rng();
            for i in 0..20 {
                let v: u8 = rng.gen();
                data[i] = v;
            }
            return NodeID { val: data };
        }

        pub fn id_clone(&self) -> [u8; NODE_ID_LENGTH] {
            self.val.clone()
        }
    }

    pub mod kademila {
        use crate::node::id::NodeID;
        use crate::node::id::NODE_ID_LENGTH;

        /// 比较两个节点的大小
        /// ## Return
        /// - 0 - 相等
        /// - 正数 - Self 更大
        /// - 负数 - Self 更小
        #[allow(unused)]
        pub fn cmp(id1: &NodeID, id2: &NodeID) -> i32 {
            let mut count = 0;
            while id1.val[count] == id2.val[count] {
                count += 1;
            }
            if count >= 20 {
                return 0;
            } else {
                return (id1.val[count] - id2.val[count]).into();
            }
        }
        #[allow(unused)]
        pub fn mid(id1: &NodeID, id2: &NodeID) -> NodeID {
            let mut node = NodeID::new();
            let mut b: u16 = 0;
            for i in (0..NODE_ID_LENGTH).rev() {
                let mid = id1.val[i] as u16 + id2.val[i] as u16 + b;
                node.val[i] = mid as u8;
                b = mid >> 8;
            }
            for i in 0..NODE_ID_LENGTH {
                let tmp = (node.val[i] as u16 + (b << 8));
                node.val[i] = (tmp / 2) as u8;
                b = tmp & 1;
            }
            return node;
        }

        pub fn lowest_bit(node: &NodeID) -> Option<usize> {
            for i in (0..NODE_ID_LENGTH).rev() {
                let v = node.val[i];
                let mut f: u8 = 1;
                for j in (0..8).rev() {
                    if v & f != 0 {
                        return Some(i * 8 + j);
                    }
                    f <<= 1;
                }
            }
            return None;
        }
    }
    #[cfg(test)]
    mod test {
        use crate::node::id::kademila::mid;

        #[test]
        fn rand_id_test() {
            let id = super::NodeID::rand();
            println!("{:?}", id);
            let id = 11 & 2;
            println!("{:?}", id);
        }

        #[test]
        fn mid_node_test() {
            let mut v1 = [0u8; 20];
            v1[18] = 0;
            v1[19] = 255;
            let id1 = super::NodeID::wrap(v1);
            let mut v2 = [0u8; 20];
            v2[18] = 1;
            v2[19] = 255;
            let id2 = super::NodeID::wrap(v2);
            let mid = mid(&id1, &id2);
            let mut v3 = [0u8; 20];
            v3[18] = 1;
            v3[19] = 127;
            assert_eq!(mid.val, v3);
        }
    }
}
