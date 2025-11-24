use std::sync::Arc;

use tokio::{io, net::TcpListener};

use crate::server::{ChatRoom, handle_connection};

mod error;
mod server;

#[tokio::main]
async fn main() -> io::Result<()> {
    let chat_room = Arc::new(ChatRoom::new());

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Listening on {}", listener.local_addr().unwrap());

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New client: {}", addr);

        let chat_room_clone = chat_room.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, chat_room_clone).await {
                eprintln!("Client connection error: {:?}", e);
            }
        });
    }
}
