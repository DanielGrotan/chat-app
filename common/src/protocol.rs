use bincode::{Decode, Encode};
use tokio::io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::uuid::Uid;

#[derive(Debug, Clone, Encode, Decode)]
pub struct ChatMessage {
    pub from: Uid,
    pub text: String,
    pub timestamp: u64,
}

#[derive(Encode, Decode, Debug, Clone)]
pub enum ClientMessage {
    Chat { text: String },
    JoinRequest { username: String },
}

#[derive(Encode, Decode, Debug, Clone)]
pub enum ServerMessage {
    Chat(ChatMessage),
    JoinAccepted {
        history: Vec<ChatMessage>,
        participants: Vec<(Uid, String)>,
    },
    UserJoined {
        uuid: Uid,
        username: String,
    },
    UserLeft {
        uuid: Uid,
    },
}

pub async fn write_msg<W: AsyncWrite + Unpin, M: Encode>(
    writer: &mut W,
    message: &M,
) -> io::Result<()> {
    let bytes = bincode::encode_to_vec(message, bincode::config::standard())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let len = bytes.len() as u32;

    writer.write_all(&len.to_be_bytes()).await?;
    writer.write_all(&bytes).await
}

pub async fn read_msg<R, M>(reader: &mut R) -> io::Result<M>
where
    R: AsyncRead + Unpin,
    M: Decode<()>,
{
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await?;

    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;

    let (message, _) = bincode::decode_from_slice(&buf, bincode::config::standard())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(message)
}
