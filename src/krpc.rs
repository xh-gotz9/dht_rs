use fraux_rs::BData;

use crate::hash::{Hash, HASH_LENGTH};
use crate::node::Node;
use crate::node::NodeID;
use core::fmt::Debug;
use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::SystemTime;

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
pub struct QPing {
    pub id: NodeID,
}

#[derive(Debug)]
pub struct QFindNode {
    pub id: NodeID,
    pub target: NodeID,
}

#[derive(Debug)]
pub struct QGetPeers {
    pub id: NodeID,
    pub info_hash: Hash,
}

#[derive(Debug)]
pub struct QAnnouncePeer {
    pub id: NodeID,
    pub info_hash: Hash,
    pub implied_port: i32,
    pub port: u16,
    pub token: Hash,
}

#[derive(Debug)]
pub enum Remote {
    Nodes(Vec<Node>),
    Peers(Vec<SocketAddr>),
}

#[derive(Debug)]
pub struct ResponseBody {
    pub id: NodeID,
    pub token: Option<Vec<u8>>,
    pub remote: Option<Remote>,
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
    if let Some(BData::Dict(r)) = dict.get("r") {
        let idv = r.get("id").and_then(|s| match s {
            BData::Number(_) | BData::List(_) | BData::Dict(_) => None,
            BData::BString(v) => Some(v.clone()),
        });

        if idv.is_none() {
            return Err("id not found".to_string());
        }

        let mut id = [0u8; HASH_LENGTH];
        id.clone_from_slice(&idv.expect("")[..HASH_LENGTH]);

        let id = Hash::wrap(id);

        let token = if let Some(BData::BString(t)) = r.get("token") {
            Some(t.clone())
        } else {
            None
        };

        // peers or nodes info
        let remote = if let Some(BData::List(l)) = r.get("values") {
            let peers: Vec<SocketAddr> = l
                .iter()
                .filter_map(|e| match e {
                    BData::BString(v) => Some(v.clone()),
                    BData::Number(_) | BData::List(_) | BData::Dict(_) => None,
                })
                .map(|e| parse_compact_net_info(&e))
                .filter_map(|e| match e {
                    Ok(addr) => Some(addr),
                    Err(_) => None,
                })
                .collect();

            if peers.is_empty() {
                None
            } else {
                Some(Remote::Peers(peers))
            }
        } else if let Some(BData::BString(v)) = r.get("nodes") {
            let mut nodes = vec![];

            let mut i: usize = 0;
            while (v.len() - i) >= COMAPCT_NODE_LENGTH {
                let node = parse_compact_node_info(&v[i..])?;
                nodes.push(node);
                i += COMAPCT_NODE_LENGTH;
            }

            if nodes.is_empty() {
                None
            } else {
                Some(Remote::Nodes(nodes))
            }
        } else {
            None
        };

        return Ok(ResponseBody { id, token, remote });
    }

    return Err("not found response body".to_string());
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

const NET_ADDRESS_LENGTH: usize = 6;

const COMAPCT_NODE_LENGTH: usize = HASH_LENGTH + NET_ADDRESS_LENGTH;

fn parse_compact_node_info(bytes: &[u8]) -> Result<Node, String> {
    if bytes.len() < COMAPCT_NODE_LENGTH {
        return Err("node info data not enough".to_string());
    }

    let mut v = [0u8; HASH_LENGTH];
    v.clone_from_slice(&bytes[..HASH_LENGTH]);
    let id = NodeID::wrap(v);

    let addr = parse_compact_net_info(&bytes[HASH_LENGTH..HASH_LENGTH + NET_ADDRESS_LENGTH])?;

    let node = Node::new(id, addr, SystemTime::now());

    Ok(node)
}

fn parse_compact_net_info(bytes: &[u8]) -> Result<SocketAddr, String> {
    if bytes.len() < 6 {
        return Err("net info data not enough".to_string());
    }

    let addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3])),
        u16::from_be_bytes([bytes[4], bytes[5]]),
    );

    Ok(addr)
}

mod tests {

    #[test]
    fn parse_get_peers_query_test() {
        let src = std::fs::read("./test/get_peers_query.dat")
            .expect("load file get_peers test data failed");

        let data = fraux_rs::parse(&src).expect("test data parse error");
        let message = super::parse_krpc_message(&data).expect("parse message failed");

        println!("{:?}", message);
    }

    #[test]
    fn parse_get_peers_response_test() {
        let src = std::fs::read("./test/get_peers_response.dat")
            .expect("load file get_peers test data failed");

        let data = fraux_rs::parse(&src).expect("test data parse error");
        let message = super::parse_krpc_message(&data).expect("parse message failed");

        println!("{:?}", message);
    }

    #[test]
    fn parse_ping_query_test() {
        let src = std::fs::read("./test/ping_query.dat")
            .expect("load file ping test data failed");

        let data = fraux_rs::parse(&src).expect("test data parse error");
        let message = super::parse_krpc_message(&data).expect("parse message failed");

        println!("{:?}", message);
    }

    #[test]
    fn parse_ping_response_test() {
        let src = std::fs::read("./test/ping_response.dat")
            .expect("load file ping test data failed");

        let data = fraux_rs::parse(&src).expect("test data parse error");
        let message = super::parse_krpc_message(&data).expect("parse message failed");

        println!("{:?}", message);
    }
        
}
