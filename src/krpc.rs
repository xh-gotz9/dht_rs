use std::{collections::BTreeMap, net::SocketAddr, rc::Rc};

use fraux_rs::BData;

use crate::hash::{Hash, HASH_LENGTH};
use crate::node::{Node, NodeID};

#[allow(unused)]
pub struct KMessage {
    transaction_id: Vec<u8>,
    request: KRequest,
}
#[allow(unused)]
pub enum KRequest {
    Query(QueryBody),
    Response(ResponseBody),
    Error((u32, String)),
}
#[allow(unused)]
pub enum QueryBody {
    Ping(QPing),
    FindNode(QFindNode),
    GetPeers(QGetPeers),
    AnnouncePeer(QAnnouncePeer),
}

#[allow(unused)]
pub enum ResponseBody {
    Ping(RPing),
    FindNode(RFindNode),
    GetPeers(RGetPeers),
    AnnouncePeer(RAnonnouncePeer),
}

#[allow(unused)]
pub struct QPing {
    pub id: NodeID,
}

pub type RPing = QPing;

#[allow(unused)]
pub struct QFindNode {
    pub id: NodeID,
    pub target: NodeID,
}

#[allow(unused)]
pub struct RFindNode {
    id: NodeID,
    token: Vec<u8>,
    nodes: Vec<Node>,
}

#[allow(unused)]
pub struct QGetPeers {
    id: NodeID,
    info_hash: Hash,
}

#[allow(unused)]
pub struct RGetPeers {
    id: NodeID,
    token: Vec<u8>,
    peers: Vec<SocketAddr>,
    nodes: Vec<Node>,
}

#[allow(unused)]
pub struct QAnnouncePeer {
    id: NodeID,
    info_hash: Hash,
    implied_port: u32,
    port: u32,
    token: Vec<u8>,
}

pub type RAnonnouncePeer = QPing;
#[allow(unused)]
pub struct Query {
    y: Vec<u8>,
}

pub fn parse_message(src: BData) -> Option<KMessage> {
    if let BData::Dict(dict) = src {
        let dict = Rc::clone(&dict);
        let transaction_id = dict
            .get("t")
            .and_then(|t| match t {
                BData::BString(s) => Some(s.clone()),
                _ => None,
            })
            .expect("no transaction id");

        if let Some(BData::BString(y)) = dict.get("y") {
            if y.len() == 1 {
                let request = match y[0] {
                    b'e' => {
                        let (no, msg) = parse_error(dict).expect("no data");
                        KRequest::Error((no, msg))
                    }
                    b'q' => {
                        let body = parse_query(dict).expect("no data");
                        KRequest::Query(body)
                    }

                    b'r' => {
                        let body = parse_response(dict).expect("no data");
                        KRequest::Response(body)
                    }
                    _ => panic!("bad request"),
                };
                return Some(KMessage {
                    transaction_id,
                    request,
                });
            }
        }
    }

    panic!("source data error")
}

pub fn parse_error(dict: Rc<BTreeMap<String, BData>>) -> Option<(u32, String)> {
    return Some((32, "STR".to_string()));
}

pub fn parse_response(dict: Rc<BTreeMap<String, BData>>) -> Option<ResponseBody> {
    Some(ResponseBody::Ping(RPing {
        id: crate::hash::Hash::rand(),
    }))
}

pub fn parse_query(dict: Rc<BTreeMap<String, BData>>) -> Option<QueryBody> {
    if let Some(BData::BString(q)) = dict.get("q") {
        let s = String::from_utf8(q.clone()).expect("string parse error");

        if let BData::Dict(addition) = dict.get("a").expect("no query body") {
            let idv = addition
                .get("id")
                .and_then(|s| match s {
                    BData::Number(_) | BData::List(_) | BData::Dict(_) => None,
                    BData::BString(v) => Some(v.clone()),
                })
                .expect("no id");

            let mut id = [0u8; HASH_LENGTH];
            id.clone_from_slice(&idv.as_slice()[..HASH_LENGTH]);

            let id = Hash::wrap(id);

            // TODO: parse query body

            if "ping".eq(&s) {
                let body = QPing { id };
                return Some(QueryBody::Ping(body));
            }

            if "find_node".eq(&s) {
                let t = addition
                    .get("id")
                    .and_then(|s| match s {
                        BData::Number(_) | BData::List(_) | BData::Dict(_) => None,
                        BData::BString(v) => Some(v.clone()),
                    })
                    .expect("no id");

                let mut target = [0u8; HASH_LENGTH];
                target.clone_from_slice(&t.as_slice()[..HASH_LENGTH]);
                let target = Hash::wrap(target);

                let body = QFindNode { id, target };
                return Some(QueryBody::FindNode(body));
            }

            if "get_peers".eq(&s) {
                let info_hash = id.clone();
                let body = QGetPeers { id, info_hash };
                return Some(QueryBody::GetPeers(body));
            }

            if "announce_peers".eq(&s) {
                let info_hash = id.clone();
                let token = id.clone().raw_id();
                let body = QAnnouncePeer {
                    id,
                    info_hash,
                    implied_port: 1,
                    port: 32,
                    token,
                };
                return Some(QueryBody::AnnouncePeer(body));
            }
        }
    }

    return None;
}
