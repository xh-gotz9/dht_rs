use std::{io, sync::Arc};

use dht_rs::{krpc::encode_message, DHTTable};
use tokio::net::UdpSocket;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> io::Result<()> {
    let receiver = Arc::new(UdpSocket::bind("0.0.0.0:8888").await?);

    let sender = Arc::clone(&receiver);
    let table = Arc::new(DHTTable::new(move |addr, message| {
        let data = encode_message(message);
        let data = fraux_rs::stringify(&data).expect("stringify failed");
        sender.clone().send_to(data.as_slice(), addr).await;
    }));

    let mut buffer = [0u8; 2048];
    loop {
        let (len, addr) = receiver.recv_from(&mut buffer).await?;
        let vec = buffer[..len].to_vec();
        let t = table.clone();

        tokio::spawn(async move {
            t.handle_message_bytes(addr, vec);
        })
        .await?;
    }
}
