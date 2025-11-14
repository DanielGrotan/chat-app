use common::protocol::{ClientMessage, ServerMessage, read_msg, write_msg};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let mut socket = TcpStream::connect("localhost:8080").await?;

    let msg = ClientMessage::JoinRequest {
        username: "My username".to_string().into(),
    };
    write_msg(&mut socket, &msg).await?;

    let msg = ClientMessage::Chat {
        text: "Hello from CLI test!".into(),
    };
    write_msg(&mut socket, &msg).await?;

    loop {
        match read_msg::<_, ServerMessage>(&mut socket).await {
            Ok(msg) => println!("Received: {:?}", msg),
            Err(e) => eprintln!("Error reading message: {:?}", e),
        }
    }
}
