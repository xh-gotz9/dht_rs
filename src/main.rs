mod bucket;
mod node;

use node::id::NodeID;

fn main() {
    let v = NodeID::rand();
    println!("{:?}", v);
}
