use std::{collections::BTreeMap, fmt::Debug, net::SocketAddr, rc::Rc};

use fraux_rs::BData;

use crate::hash::{Hash, HASH_LENGTH};
use crate::node::{Node, NodeID};

#[allow(unused)]
pub struct KMessage {
    pub transaction_id: Vec<u8>,
    pub request: KRequest,
}

impl Debug for KMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KMessage")
            .field("transaction_id", &hex::encode_upper(&self.transaction_id))
            .field("request", &self.request)
            .finish()
    }
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
    pub nodes: Vec<Rc<Node>>,
}

#[allow(unused)]
#[derive(Debug)]
pub struct QAnnouncePeer {
    pub id: NodeID,
    pub info_hash: Hash,
    pub implied_port: i32,
    pub port: u16,
    pub token: Vec<u8>,
}

pub type RAnonnouncePeer = QPing;

#[allow(unused)]
#[derive(Debug)]
pub struct Query {
    y: Vec<u8>,
}

pub fn decode_message(src: BData) -> Option<KMessage> {
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
                        let (no, msg) = decode_error(dict).expect("no data");
                        KRequest::Error((no, msg))
                    }
                    b'q' => {
                        let body = decode_query(dict).expect("no data");
                        KRequest::Query(body)
                    }

                    b'r' => {
                        let body = decode_response(dict).expect("no data");
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

fn decode_error(_dict: Rc<BTreeMap<String, BData>>) -> Option<(u32, String)> {
    todo!("decode error message data")
}

fn decode_response(_dict: Rc<BTreeMap<String, BData>>) -> Option<ResponseBody> {
    todo!("decode response message data")
}

fn decode_query(dict: Rc<BTreeMap<String, BData>>) -> Option<QueryBody> {
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
                    let info = addition
                        .get("info_hash")
                        .and_then(|h| match h {
                            BData::BString(s) => Some(s.clone()),
                            BData::Number(_) | BData::List(_) | BData::Dict(_) => None,
                        })
                        .expect("no info_hash");

                    // todo: info length != 20, 则应视为异常数据
                    let mut arr = [0u8; HASH_LENGTH];
                    arr.clone_from_slice(&info.as_slice()[..HASH_LENGTH]);
                    let info_hash = Hash::wrap(arr);

                    let body = QGetPeers { id, info_hash };
                    return Some(QueryBody::GetPeers(body));
                }
                "announce_peers" => {
                    // todo: implement read params
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

#[allow(unused)]
fn encode_message(message: KMessage) -> BData {
    let mut map = BTreeMap::new();

    // transaction id
    map.insert("t".to_string(), BData::BString(message.transaction_id));

    // y
    match message.request {
        KRequest::Query(body) => {
            map.insert("y".to_string(), BData::BString("q".as_bytes().to_vec()));
            map.append(&mut encode_query(body));
        }
        KRequest::Response(body) => {
            map.insert("y".to_string(), BData::BString("r".as_bytes().to_vec()));
            map.append(&mut encode_response(body));
        }
        KRequest::Error(body) => {
            map.insert("y".to_string(), BData::BString("e".as_bytes().to_vec()));
        }
    }

    BData::Dict(Rc::new(map))
}

#[allow(unused)]
fn encode_query(query: QueryBody) -> BTreeMap<String, BData> {
    let mut map = BTreeMap::new();

    match query {
        QueryBody::Ping(body) => {
            map.insert("q".to_string(), BData::BString("ping".as_bytes().to_vec()));

            // encode body
            let mut a = BTreeMap::new();
            a.insert("id".to_string(), BData::BString(body.id.raw_id()));

            map.insert("a".to_string(), BData::Dict(Rc::new(a)));
        }
        QueryBody::FindNode(body) => {
            map.insert("q".to_string(), BData::BString("ping".as_bytes().to_vec()));

            // encode body
            let mut a = BTreeMap::new();
            a.insert("id".to_string(), BData::BString(body.id.raw_id()));
            a.insert("target".to_string(), BData::BString(body.target.raw_id()));

            map.insert("a".to_string(), BData::Dict(Rc::new(a)));
        }
        QueryBody::GetPeers(body) => {
            map.insert("q".to_string(), BData::BString("ping".as_bytes().to_vec()));

            // encode body
            let mut a = BTreeMap::new();
            a.insert("id".to_string(), BData::BString(body.id.raw_id()));
            a.insert(
                "info_hash".to_string(),
                BData::BString(body.info_hash.raw_id()),
            );

            map.insert("a".to_string(), BData::Dict(Rc::new(a)));
        }
        QueryBody::AnnouncePeer(body) => {
            map.insert("q".to_string(), BData::BString("ping".as_bytes().to_vec()));

            // encode body
            let mut a = BTreeMap::new();
            a.insert("id".to_string(), BData::BString(body.id.raw_id()));
            a.insert(
                "implied_port".to_string(),
                BData::Number(body.implied_port.into()),
            );
            a.insert(
                "info_hash".to_string(),
                BData::BString(body.info_hash.raw_id()),
            );
            a.insert("port".to_string(), BData::Number(body.port.into()));
            a.insert("token".to_string(), BData::BString(body.token));

            map.insert("a".to_string(), BData::Dict(Rc::new(a)));
        }
    }

    map
}

#[allow(unused)]
fn encode_response(body: ResponseBody) -> BTreeMap<String, BData> {
    let mut map = BTreeMap::new();

    match body {
        ResponseBody::Ping(body) => {
            // encode body
            let mut r = BTreeMap::new();
            r.insert("id".to_string(), BData::BString(body.id.raw_id()));

            map.insert("r".to_string(), BData::Dict(Rc::new(r)));
        }

        ResponseBody::FindNode(body) => {
            // encode body
            let mut r = BTreeMap::new();
            r.insert("id".to_string(), BData::BString(body.id.raw_id()));
            let mut v = vec![];
            body.nodes.iter().for_each(|node| {
                v.append(&mut node.compacted_info());
            });
            r.insert("nodes".to_string(), BData::BString(v));

            map.insert("r".to_string(), BData::Dict(Rc::new(r)));
        }

        ResponseBody::GetPeers(body) => {
            // encode body
            let mut r = BTreeMap::new();
            r.insert("id".to_string(), BData::BString(body.id.raw_id()));
            r.insert("token".to_string(), BData::BString(body.token));
            let mut info = vec![];
            body.nodes.iter().for_each(|node| {
                info.append(&mut node.compacted_info());
            });
            r.insert("nodes".to_string(), BData::BString(info));

            map.insert("r".to_string(), BData::Dict(Rc::new(r)));
        }

        ResponseBody::AnnouncePeer(body) => {
            // encode body
            let mut r = BTreeMap::new();
            r.insert("id".to_string(), BData::BString(body.id.raw_id()));

            map.insert("r".to_string(), BData::Dict(Rc::new(r)));
        }
    }

    map
}
