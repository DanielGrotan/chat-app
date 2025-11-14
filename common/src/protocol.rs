use std::sync::Arc;

use bincode::{Decode, Encode, config};
use bytes::{BufMut, Bytes, BytesMut};
use tokio::io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::uuid::Uid;

#[derive(Debug, Clone, Encode, Decode)]
pub struct ChatMessage {
    pub from: Uid,
    pub text: Arc<str>,
    pub timestamp: u64,
}

#[derive(Encode, Decode, Debug, Clone)]
pub enum ClientMessage {
    Chat { text: Arc<str> },
    JoinRequest { username: Arc<str> },
}

#[derive(Encode, Decode, Debug, Clone)]
pub enum ServerMessage {
    Chat(Arc<ChatMessage>),
    JoinAccepted {
        history: Vec<Arc<ChatMessage>>,
        participants: Vec<(Uid, Arc<str>)>,
    },
    UserJoined {
        uuid: Uid,
        username: Arc<str>,
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

    let mut data = vec![0u8; len];
    reader.read_exact(&mut data).await?;

    let config = config::standard();

    let (message, _) = bincode::decode_from_slice(&data, config)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(message)
}

pub async fn encode_message<T: Encode>(message: &T) -> io::Result<Bytes> {
    let config = config::standard();

    let data = bincode::encode_to_vec(message, config)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut buf = BytesMut::with_capacity(4 + data.len());

    buf.put_u32(data.len() as u32);
    buf.extend_from_slice(&data);

    Ok(buf.freeze())
}
