use dht_rs::{kademlia::KademliaTable, node::Node};

fn main() {
    let table = KademliaTable::new();

    for _i in 0..9 {
        table.insert_node(Node::random())
    }
}
