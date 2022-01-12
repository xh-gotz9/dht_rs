use fraux_rs::BData;

use crate::hash::{Hash, HASH_LENGTH};
use crate::node::Node;
use crate::node::NodeID;
use core::fmt::Debug;
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::rc::Rc;

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

#[derive(Debug)]
pub enum KRequest {
    Query(QueryBody),
    Response(ResponseBody),
    Error((u32, String)),
}

#[derive(Debug)]
pub enum QueryBody {
    Ping(QPing),
    FindNode(QFindNode),
    GetPeers(QGetPeers),
    AnnouncePeer(QAnnouncePeer),
}

#[derive(Debug)]
pub enum ResponseBody {
    Ping(RPing),
    FindNode(RFindNode),
    GetPeers(RGetPeers),
    AnnouncePeer(RAnonnouncePeer),
}

#[derive(Debug)]
pub struct QPing {
    pub id: NodeID,
}

pub type RPing = QPing;

#[derive(Debug)]
pub struct QFindNode {
    pub id: NodeID,
    pub target: NodeID,
}

#[derive(Debug)]
pub struct RFindNode {
    pub id: NodeID,
    pub nodes: Vec<Node>,
}

#[derive(Debug)]
pub struct QGetPeers {
    pub id: NodeID,
    pub info_hash: Hash,
}

#[derive(Debug)]
pub struct RGetPeers {
    pub id: NodeID,
    pub token: Vec<u8>,
    pub peers: Vec<SocketAddr>,
    pub nodes: Vec<Rc<Node>>,
}

#[derive(Debug)]
pub struct QAnnouncePeer {
    pub id: NodeID,
    pub info_hash: Hash,
    pub implied_port: i32,
    pub port: u16,
    pub token: Hash,
}

pub type RAnonnouncePeer = QPing;

#[derive(Debug)]
pub struct Query {
    y: Hash,
}

pub fn parse_krpc_message(src: &BData) -> Result<KMessage, String> {
    if let BData::Dict(d) = src {
        let transaction_id = d.get("t").and_then(|t| match t {
            BData::BString(s) => Some(s.clone()),
            BData::Number(_) | BData::List(_) | BData::Dict(_) => None,
        });

        if let None = transaction_id {
            return Err("not found transaction id".to_string());
        }

        let transaction_id = transaction_id.unwrap();

        if let Some(BData::BString(y)) = d.get("y") {
            if y.len() > 1 {
                return Err("excepted 'y' data".to_string());
            }

            let r = match y[0] {
                b'e' => parse_error(&d).map(|e| KRequest::Error(e)),
                b'q' => parse_query(&d).map(|query| KRequest::Query(query)),
                b'r' => parse_response(&d).map(|response| KRequest::Response(response)),
                _ => Err("unexcepted 'y' value".to_string()),
            };

            return r.map(|request| KMessage {
                transaction_id,
                request,
            });
        } else {
            return Err("excepted 'y' type".to_string());
        }
    }

    Err("failed".to_string())
}

fn parse_error(dict: &BTreeMap<String, BData>) -> Result<(u32, String), String> {
    Err("".to_string())
}

fn parse_response(dict: &BTreeMap<String, BData>) -> Result<ResponseBody, String> {
    todo!("decode response message data")
}

fn parse_query(dict: &BTreeMap<String, BData>) -> Result<QueryBody, String> {
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
            return match s.as_ref() {
                "ping" => {
                    let body = QPing { id };

                    Ok(QueryBody::Ping(body))
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

                    Ok(QueryBody::FindNode(body))
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

                    println!("{:?}", arr);

                    let body = QGetPeers { id, info_hash };

                    Ok(QueryBody::GetPeers(body))
                }
                "announce_peers" => {
                    // todo: implement read params
                    let info_hash = id.clone();
                    let token = id.clone();
                    let body = QAnnouncePeer {
                        id,
                        info_hash,
                        implied_port: 1,
                        port: 32,
                        token,
                    };

                    Ok(QueryBody::AnnouncePeer(body))
                }
                _ => Err("".to_string()),
            };
        }
    }

    return Err("data struct error".to_string());
}
