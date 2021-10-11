use dht_rs::node::Node;

fn main() {
    let node = Node::random();
    println!("{:?}", node);
}
