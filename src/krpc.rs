use std::{collections::BTreeMap, net::SocketAddr, rc::Rc};

use fraux_rs::BData;

use crate::hash::{Hash, HASH_LENGTH};
use crate::node::{Node, NodeID};

#[allow(unused)]
#[derive(Debug)]
pub struct KMessage {
    pub transaction_id: Vec<u8>,
    pub request: KRequest,
}
#[allow(unused)]
#[derive(Debug)]
pub enum KRequest {
    Query(QueryBody),
    Response(ResponseBody),
    Error((u32, String)),
}
#[allow(unused)]
#[derive(Debug)]
pub enum QueryBody {
    Ping(QPing),
    FindNode(QFindNode),
    GetPeers(QGetPeers),
    AnnouncePeer(QAnnouncePeer),
}

#[allow(unused)]
#[derive(Debug)]
pub enum ResponseBody {
    Ping(RPing),
    FindNode(RFindNode),
    GetPeers(RGetPeers),
    AnnouncePeer(RAnonnouncePeer),
}

#[allow(unused)]
#[derive(Debug)]
pub struct QPing {
    pub id: NodeID,
}

pub type RPing = QPing;

#[allow(unused)]
#[derive(Debug)]
pub struct QFindNode {
    pub id: NodeID,
    pub target: NodeID,
}

#[allow(unused)]
#[derive(Debug)]
pub struct RFindNode {
    pub id: NodeID,
    pub nodes: Vec<Node>,
}

#[allow(unused)]
#[derive(Debug)]
pub struct QGetPeers {
    pub id: NodeID,
    pub info_hash: Hash,
}

#[allow(unused)]
#[derive(Debug)]
pub struct RGetPeers {
    pub id: NodeID,
    pub token: Vec<u8>,
    pub peers: Vec<SocketAddr>,
    pub nodes: Vec<Node>,
}

#[allow(unused)]
#[derive(Debug)]
pub struct QAnnouncePeer {
    pub id: NodeID,
    pub info_hash: Hash,
    pub implied_port: u32,
    pub port: u32,
    pub token: Vec<u8>,
}

pub type RAnonnouncePeer = QPing;

#[allow(unused)]
#[derive(Debug)]
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

            // parse query body
            match s.as_ref() {
                "ping" => {
                    let body = QPing { id };
                    return Some(QueryBody::Ping(body));
                }
                "find_node" => {
                    let t = addition
                        .get("target")
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

                "get_peers" => {
                    let info_hash = id.clone();
                    let body = QGetPeers { id, info_hash };
                    return Some(QueryBody::GetPeers(body));
                }
                "announce_peers" => {
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
                _ => {}
            }
        }
    }
    return None;
}
