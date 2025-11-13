use bincode::{Decode, Encode};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Encode, Decode, Debug)]
pub struct ChatMessage {
    pub from: String,
    pub text: String,
}

#[derive(Encode, Decode, Debug)]
pub enum Message {
    Chat(ChatMessage),
}

pub async fn write_msg(socket: &mut TcpStream, message: &Message) -> io::Result<()> {
    let bytes = bincode::encode_to_vec(message, bincode::config::standard())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let len = bytes.len() as u32;

    socket.write_all(&len.to_be_bytes()).await?;
    socket.write_all(&bytes).await
}

pub async fn read_msg(socket: &mut TcpStream) -> io::Result<Message> {
    let mut len_buf = [0u8; 4];
    socket.read_exact(&mut len_buf).await?;

    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    socket.read_exact(&mut buf).await?;

    let (message, _) = bincode::decode_from_slice(&buf, bincode::config::standard())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(message)
}
