pub mod bucket;
pub mod hash;
pub mod krpc;
pub mod node;

use crate::krpc::*;
use crate::node::Node;
use std::{cell::RefCell, rc::Rc};
use std::{net::SocketAddr, time::SystemTime};

use bucket::Bucket;
use hash::{Hash, HASH_LENGTH};
use krpc::{KMessage, QAnnouncePeer, QFindNode, QGetPeers, QPing};
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

    #[allow(unused)]
    fn handle_message_bytes(&self, addr: SocketAddr, src: Vec<u8>) {
        let msg = fraux_rs::parse(src).expect("bencode 解析失败");
        let message = krpc::decode_message(msg).expect("krpc 解析失败");
        self.handle_message(addr, message);
    }

    #[allow(unused)]
    fn handle_message(&self, addr: SocketAddr, mut message: KMessage) {
        match &message.request {
            krpc::KRequest::Query(query) => {
                let resp = match query {
                    krpc::QueryBody::Ping(query) => self.handle_q_ping(&query),
                    krpc::QueryBody::FindNode(query) => self.handle_q_find_node(&query),
                    krpc::QueryBody::GetPeers(query) => self.handle_q_get_peers(&query),
                    krpc::QueryBody::AnnouncePeer(query) => self.handle_q_announce_peers(&query),
                };

                if let Some(res) = resp {
                    message.request = KRequest::Response(res);
                    self.send_response(addr, &message);
                }
            }
            krpc::KRequest::Response(response) => match response {
                _ => todo!("handle response"),
            },
            krpc::KRequest::Error(error) => todo!("handle error"),
        }
    }

    #[allow(unused)]
    fn send_response(&self, addr: SocketAddr, body: &KMessage) {
        // todo: implement send message
        println!("address: {:?}, content: {:?}", addr, body);
    }

    /// 处理 krpc 的 ping 请求
    #[allow(unused)]
    fn handle_q_ping(&self, query: &QPing) -> Option<ResponseBody> {
        let bucket_ref = self.search_bucket(&query.id);
        if let Some(b) = bucket_ref {
            let mut b = b.as_ref().borrow_mut();

            if let Some(node) = b.find_node(&query.id) {
                // todo: update node life info
            } else {
                b.insert_node(Node::new(
                    query.id.clone(),
                    "0.0.0.0:3333".parse().expect("socket addr parse error"),
                    SystemTime::now(),
                ))
            }
            return Some(ResponseBody::Ping(RPing {
                id: self.id.clone(),
            }));
        }
        // ignore
        None
    }

    /// 处理 krpc 的 ping 响应
    #[allow(unused)]
    fn handle_r_ping(&self, response: &RPing) {
        if let Some(b) = self
            .search_bucket(&response.id)
            .and_then(|b| b.as_ref().borrow().find_node(&response.id))
        {
            todo!("update node life info")
        }
    }

    /// 处理 krpc 的 find_node 请求
    #[allow(unused)]
    fn handle_q_find_node(&self, query: &QFindNode) -> Option<ResponseBody> {
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

    #[allow(unused)]
    fn handle_r_find_node(&self, resp: &RFindNode) {
        // add nodes in response to dht table
        todo!("handle find_node response")
    }

    /// 处理 krpc 的 get_peers 请求
    /// 暂不处理
    #[allow(unused)]
    fn handle_q_get_peers(&self, query: &QGetPeers) -> Option<ResponseBody> {
        None
    }

    #[allow(unused)]
    fn handle_r_get_peers(&self, resp: &RGetPeers) {
        todo!("handle get_peers response")
    }

    /// 处理 krpc 的 announce_peers 请求
    /// 暂不处理
    #[allow(unused)]
    fn handle_q_announce_peers(&self, query: &QAnnouncePeer) -> Option<ResponseBody> {
        None
    }

    #[allow(unused)]
    fn handle_r_announce_peers(&self, resp: &RAnonnouncePeer) {
        todo!("handle get_peers response")
    }
}

#[cfg(test)]
mod test {
    use crate::{
        hash::Hash,
        krpc::{KMessage, KRequest, QFindNode, QueryBody},
        DHTTable,
    };
    use crate::{node::NodeID, QPing};

    #[test]
    fn handle_query_ping() {
        let table = DHTTable::new();

        let ping = QPing { id: NodeID::rand() };

        let message = KMessage {
            transaction_id: "tt".as_bytes().to_vec(),
            request: KRequest::Query(QueryBody::Ping(ping)),
        };

        table.handle_message(
            "0.0.0.0:3333".parse().expect("socket addr parse error"),
            message,
        );
    }

    #[test]
    fn handle_query_find_node() {
        let table = DHTTable::new();

        let body = QFindNode {
            id: NodeID::rand(),
            target: Hash::rand(),
        };

        let message = KMessage {
            transaction_id: "tt".as_bytes().to_vec(),
            request: KRequest::Query(QueryBody::FindNode(body)),
        };

        table.handle_message(
            "0.0.0.0:3333".parse().expect("socket addr parse error"),
            message,
        )
    }
}
