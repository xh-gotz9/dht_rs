use std::{net::SocketAddr, time::SystemTime};

pub use id::NodeID;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Node {
    pub id: NodeID,
    addr: SocketAddr,
    last_change: SystemTime,
}

impl Node {
    #[allow(unused)]
    pub fn new(id: NodeID, addr: SocketAddr, last_change: SystemTime) -> Self {
        Self {
            id,
            addr,
            last_change,
        }
    }

    #[allow(unused)]
    pub fn is_good_node(&self) -> bool {
        self.last_change
            .elapsed()
            .and_then(|x| Ok(x.as_secs() < 60 * 15))
            .unwrap_or(false)
    }
}

pub mod id {

    pub const NODE_ID_LENGTH: usize = crate::hash::HASH_LENGTH;

    pub type NodeID = crate::hash::Hash;

    /// 比较两个节点的大小
    /// ## Return
    /// - 0 - 相等
    /// - 正数 - Self 更大
    /// - 负数 - Self 更小
    #[allow(unused)]
    pub fn cmp(id1: &NodeID, id2: &NodeID) -> i32 {
        let mut count = 0;
        while id1.val[count] == id2.val[count] {
            count += 1;
        }
        if count >= 20 {
            return 0;
        } else {
            return (id1.val[count] as i32 - id2.val[count] as i32) as i32;
        }
    }

    #[allow(unused)]
    pub fn mid(id1: &NodeID, id2: &NodeID) -> NodeID {
        let mut node = NodeID::new();
        let mut b: u16 = 0;
        for i in (0..NODE_ID_LENGTH).rev() {
            let mid = id1.val[i] as u16 + id2.val[i] as u16 + b;
            node.val[i] = mid as u8;
            b = mid >> 8;
        }
        for i in 0..NODE_ID_LENGTH {
            let tmp = (node.val[i] as u16 + (b << 8));
            node.val[i] = (tmp / 2) as u8;
            b = tmp & 1;
        }
        return node;
    }

    #[allow(unused)]
    pub fn lowest_bit(node: &NodeID) -> Option<usize> {
        let mut ret: Option<usize> = None;
        for i in (0..NODE_ID_LENGTH).rev() {
            let v = node.val[i];
            let mut f: u8 = 1;
            for j in (0..8).rev() {
                if v & f != 0 {
                    ret = Some(i * 8 + j);
                    break;
                }
                f <<= 1;
            }
        }
        return ret;
    }

    #[cfg(test)]
    mod test {
        use crate::node::id::mid;

        #[test]
        fn rand_id_test() {
            let id = super::NodeID::rand();
            println!("{:?}", id);
            let id = 11 & 2;
            println!("{:?}", id);
        }

        #[test]
        fn mid_node_test() {
            let mut v1 = [0u8; 20];
            v1[18] = 0;
            v1[19] = 255;
            let id1 = super::NodeID::wrap(v1);
            let mut v2 = [0u8; 20];
            v2[18] = 1;
            v2[19] = 255;
            let id2 = super::NodeID::wrap(v2);
            let mid = mid(&id1, &id2);
            let mut v3 = [0u8; 20];
            v3[18] = 1;
            v3[19] = 127;
            assert_eq!(mid.val, v3);
        }
    }
}
