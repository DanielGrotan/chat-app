use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use bytes::Bytes;
use common::{
    protocol::{ChatMessage, ClientMessage, read_msg},
    uuid::Uid,
};
use tokio::{
    io::AsyncWriteExt,
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::mpsc::{self, UnboundedReceiver},
};

use crate::{
    error::{Error, Result},
    server::{chat_room::ChatRoom, participant::Participant},
};

pub async fn handle_connection(socket: TcpStream, chat_room: Arc<ChatRoom>) -> Result<()> {
    let (mut reader, writer) = socket.into_split();

    let (username, rx, uuid) = handle_room_join(&mut reader, &chat_room).await?;

    let result = tokio::select! {
        res = read_messages(reader, &chat_room, &uuid, username) => res,
        res = write_messages(rx, writer) => res
    };

    chat_room.leave(&uuid).await?;
    result
}

async fn handle_room_join(
    reader: &mut OwnedReadHalf,
    chat_room: &ChatRoom,
) -> Result<(Arc<str>, UnboundedReceiver<Bytes>, Uid)> {
    let username = match read_msg(reader).await {
        Ok(ClientMessage::JoinRequest { username }) => username,
        _ => return Err(Error::FailedToJoin),
    };

    let (tx, rx) = mpsc::unbounded_channel::<Bytes>();
    let participant = Participant::new(username.clone(), tx);
    let uuid = Uid::new();

    chat_room.join(&uuid, participant).await?;

    Ok((username, rx, uuid))
}

async fn read_messages(
    mut reader: OwnedReadHalf,
    chat_room: &ChatRoom,
    user_uuid: &Uid,
    username: Arc<str>,
) -> Result<()> {
    loop {
        let message = read_msg::<_, ClientMessage>(&mut reader)
            .await
            .map_err(|_| Error::ConnectionClosed {
                uuid: user_uuid.clone(),
                username: username.clone(),
            })?;

        match message {
            ClientMessage::Chat { text } => {
                let message = ChatMessage {
                    from: user_uuid.clone(),
                    text,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };
                chat_room.relay_message(message, user_uuid).await?;
            }
            ClientMessage::JoinRequest { username: _ } => {
                return Err(Error::AlreadyJoined {
                    uuid: user_uuid.clone(),
                    username,
                });
            }
        };
    }
}

async fn write_messages(
    mut rx: UnboundedReceiver<Bytes>,
    mut writer: OwnedWriteHalf,
) -> Result<()> {
    while let Some(message) = rx.recv().await {
        if writer.write_all(&message).await.is_err() {
            break;
        }
    }

    Ok(())
}
