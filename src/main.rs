mod bucket;
mod node;

use fraux_rs::BData;
use std::net::{SocketAddr, SocketAddrV4, UdpSocket};
use std::{collections::BTreeMap, rc::Rc};

use node::id::NodeID;

// router.utorrent.com
const DHT_BOOT_NODE_IP: &str = "135.216.109.124";
const DHT_BOOT_NODE_PORT: u16 = 8526;

fn main() -> std::io::Result<()> {
    let mut id = NodeID::rand();
    let mut target_id = NodeID::rand();
    println!("{:?}", id);
    let mut v = BTreeMap::new();
    let t: u16 = rand::random();
    v.insert(
        String::from("t"),
        BData::BString(hex::encode([(t >> 8) as u8, t as u8])),
    );
    v.insert(String::from("y"), BData::BString(String::from("q")));
    v.insert(String::from("q"), BData::BString(String::from("get_peers")));

    // "a" args
    let mut args = BTreeMap::new();
    args.insert(String::from("id"), BData::BString(id.raw_id()));
    args.insert(
        String::from("info_hash"),
        BData::BString(target_id.raw_id()),
    );
    v.insert(String::from("a"), BData::Dict(Rc::new(args)));

    let dict = BData::Dict(Rc::new(v));

    let data = fraux_rs::stringify(&dict).expect("stringify err");

    // 发送
    let node_addr = SocketAddrV4::new(
        DHT_BOOT_NODE_IP.parse().expect("node addr parse failed"),
        DHT_BOOT_NODE_PORT,
    );

    println!("request:\n{}", hex::encode(data.as_bytes()));

    let socket = UdpSocket::bind("0.0.0.0:9000")?;
    socket.send_to(data.as_bytes(), node_addr)?;

    let mut buffer = [0u8; 2048];
    let mut count = 0;
    while count < 1 {
        let (len, addr) = socket.recv_from(&mut buffer)?;
        println!("received data: {} bytes", len);
        let buffer = &buffer[..len];
        if addr.eq(&SocketAddr::V4(node_addr)) {
            let response =
                String::from_utf8(buffer.to_vec()).expect("response parse string failed");
            let res =
                fraux_rs::parse(&response).unwrap_or(BData::BString("parse failed".to_string()));
            println!("response:\n{:?}", res);
            count += 1;
        }
    }

    Ok(())
}
