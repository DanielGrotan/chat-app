use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use common::{
    protocol::{ChatMessage, ClientMessage, ServerMessage, read_msg, write_msg},
    uuid::Uid,
};
use tokio::{
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
    let (mut reader, mut writer) = socket.into_split();

    let username = handle_room_join(&mut reader, &mut writer, chat_room.clone()).await?;
    let (tx, rx) = mpsc::unbounded_channel::<Arc<ServerMessage>>();
    let participant = Participant::new(username.clone(), tx);

    let uuid = Uid::new();

    chat_room.join(&uuid, participant).await;

    let result = tokio::select! {
        res = read_messages(reader, chat_room.clone(), &uuid, &username) => res,
        res = write_messages(rx, writer) => res
    };

    chat_room.leave(&uuid).await;
    result
}

async fn handle_room_join(
    reader: &mut OwnedReadHalf,
    writer: &mut OwnedWriteHalf,
    chat_room: Arc<ChatRoom>,
) -> Result<String> {
    let username = match read_msg(reader).await {
        Ok(ClientMessage::JoinRequest { username }) => Ok(username),
        _ => Err(Error::FailedToJoin),
    }?;

    let join_accepted = ServerMessage::JoinAccepted {
        history: chat_room.get_history().await,
        participants: chat_room.get_usernames().await,
    };
    let _ = write_msg(writer, &join_accepted).await;

    Ok(username)
}

async fn read_messages(
    mut reader: OwnedReadHalf,
    chat_room: Arc<ChatRoom>,
    user_uuid: &Uid,
    username: &str,
) -> Result<()> {
    loop {
        let message = read_msg::<_, ClientMessage>(&mut reader)
            .await
            .map_err(|_| Error::ConnectionClosed {
                uuid: user_uuid.clone(),
                username: username.to_string(),
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
                chat_room.relay_message(message, user_uuid).await;
            }
            ClientMessage::JoinRequest { username: _ } => {
                return Err(Error::AlreadyJoined {
                    uuid: user_uuid.clone(),
                    username: username.to_string(),
                });
            }
        };
    }
}

async fn write_messages(
    mut rx: UnboundedReceiver<Arc<ServerMessage>>,
    mut writer: OwnedWriteHalf,
) -> Result<()> {
    while let Some(message) = rx.recv().await {
        write_msg(&mut writer, &message)
            .await
            .map_err(|_| Error::EncodeError { message })?;
    }

    Ok(())
}
