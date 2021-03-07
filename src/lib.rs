pub mod bucket;
pub mod hash;
pub mod krpc;
pub mod node;

use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use bucket::Bucket;
use hash::{Hash, HASH_LENGTH};
use krpc::{QFindNode, QPing};
use node::NodeID;

pub struct DHTTable {
    buckets: Option<Rc<RefCell<Bucket>>>,
}

impl DHTTable {
    #[allow(unused)]
    fn new() -> Self {
        let id = Hash::wrap([0; HASH_LENGTH]);
        Self {
            buckets: Some(Rc::new(RefCell::new(Bucket::new(id, None)))),
        }
    }

    /// 根据提供的 id 找到对应的 bucket
    fn search_bucket(&self, id: &NodeID) -> Option<Rc<RefCell<Bucket>>> {
        let mut bu = self.buckets.as_ref().map(Rc::clone);

        while let Some(b) = bu.as_ref() {
            let rc = Rc::clone(b);
            let v = rc.as_ref().borrow();

            if node::id::cmp(v.node_id(), id) <= 0 {
                return Some(Rc::clone(&rc));
            }

            bu = rc
                .as_ref()
                .borrow()
                .next_bucket()
                .as_ref()
                .and_then(|rc| Some(Rc::clone(rc)));
        }

        None
    }

    /// 处理 krpc 的 ping 请求
    #[allow(unused)]
    fn handle_ping(&self, query: &QPing) {
        let node = self.search_bucket(&query.id);
        if let Some(n) = node {
            todo!("response: pong");
        }
        // ignore
    }

    /// 处理 krpc 的 find_node 请求
    #[allow(unused)]
    fn handle_find_node(&self, query: &QFindNode) {
        let id = &query.id;
        let b = self.buckets.borrow();

        while let Some(cell) = b {
            let bu = cell.as_ref().borrow();
            if (node::id::cmp(id, bu.node_id()) >= 0) {
                // process query
                if let Some(node) = bu.find_node(id) {
                    todo!("response node data")
                } else {
                    todo!("response bucket data")
                }
                break;
            } else {
                let b = cell.as_ref().borrow().next_bucket();
            }
        }

    }

    /// 处理 krpc 的 get_peers 请求
    #[allow(unused)]
    fn handle_get_peers() {}

    /// 处理 krpc 的 announce_peers 请求
    #[allow(unused)]
    fn handle_announce_peers() {}
}
