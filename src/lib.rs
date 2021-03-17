pub mod bucket;
pub mod hash;
pub mod krpc;
pub mod node;

use crate::krpc::RFindNode;
use crate::krpc::RPing;
use crate::krpc::ResponseBody;
use std::{cell::RefCell, rc::Rc};

use bucket::Bucket;
use hash::{Hash, HASH_LENGTH};
use krpc::{QFindNode, QPing};
use node::NodeID;

pub struct DHTTable {
    id: NodeID,
    buckets: Option<Rc<RefCell<Bucket>>>,
}

impl DHTTable {
    #[allow(unused)]
    fn new() -> Self {
        let id = Hash::wrap([0; HASH_LENGTH]);
        Self {
            id: NodeID::rand(),
            buckets: Some(Rc::new(RefCell::new(Bucket::new(id, None)))),
        }
    }

    /// 根据提供的 id 找到对应的 bucket
    #[allow(unused)]
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
    fn handle_ping(&self, query: &QPing) -> Option<ResponseBody> {
        let bucket_ref = self.search_bucket(&query.id);
        if let Some(b) = bucket_ref {
            if let Some(node) = b.as_ref().borrow().find_node(&query.id) {
                return Some(ResponseBody::Ping(RPing {
                    id: self.id.clone(),
                }));
            }
        }
        // ignore
        None
    }

    /// 处理 krpc 的 find_node 请求
    #[allow(unused)]
    fn handle_find_node(&self, query: &QFindNode) -> Option<ResponseBody> {
        let id = &query.id;

        let b = self.search_bucket(id);
        if let Some(cell) = b {
            let bu = cell.as_ref().borrow();
            if (node::id::cmp(id, bu.node_id()) >= 0) {
                // process query
                let mut response;

                if let Some(node) = bu.find_node(id) {
                    response = RFindNode {
                        id: self.id.clone(),
                        nodes: vec![node.as_ref().clone()],
                    };
                } else {
                    response = RFindNode {
                        id: self.id.clone(),
                        nodes: vec![],
                    };
                }

                return Some(ResponseBody::FindNode(response));
            }
        }
        None
    }

    /// 处理 krpc 的 get_peers 请求
    #[allow(unused)]
    fn handle_get_peers() {}

    /// 处理 krpc 的 announce_peers 请求
    #[allow(unused)]
    fn handle_announce_peers() {}
}
