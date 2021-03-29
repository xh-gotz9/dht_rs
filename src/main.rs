use std::{io, sync::Arc};

use dht_rs::DHTTable;
use tokio::net::UdpSocket;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> io::Result<()> {
    let table = Arc::new(DHTTable::new());

    let sock = UdpSocket::bind("0.0.0.0:8888").await?;

    let mut buffer = [0u8; 2048];
    {
        let (len, addr) = sock.recv_from(&mut buffer).await?;
        let vec = buffer[..len].to_vec();
        let t = table.clone();
        tokio::spawn(async move {
            t.handle_message_bytes(addr, vec);
        })
        .await;
    }

    Ok(())
}
