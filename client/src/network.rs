use common::protocol::{ClientMessage, ServerMessage, read_msg, write_msg};
use tokio::{
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};

use crate::{
    error::{Error, Result},
    message::{NetworkMessage, UiMessage},
};

pub async fn handle_networking(
    tx: UnboundedSender<NetworkMessage>,
    mut rx: UnboundedReceiver<UiMessage>,
) -> Result<()> {
    let socket = loop {
        match join_room(&mut rx, &tx).await {
            Ok(socket) => break socket,
            Err(Error::InvalidAddress) => {
                tx.send(NetworkMessage::InvalidAddress)
                    .map_err(|_| Error::ChannelClosed)?;
                continue;
            }
            Err(e) => return Err(e),
        }
    };

    let (reader, writer) = socket.into_split();

    tokio::select! {
        res = read_from_ui(rx, writer) => { res? },
        res = write_to_ui(tx, reader) => { res? }
    }

    Ok(())
}

async fn join_room(
    rx: &mut UnboundedReceiver<UiMessage>,
    tx: &UnboundedSender<NetworkMessage>,
) -> Result<TcpStream> {
    let (address, username) = match rx.recv().await {
        Some(UiMessage::JoinRoom { address, username }) => (address, username),
        Some(_) => return Err(Error::ProtocolError),
        None => return Err(Error::ChannelClosed),
    };

    let address = address.trim();
    let username = username.trim();

    if address.is_empty() || username.is_empty() {
        return Err(Error::InvalidAddress);
    }

    let mut socket = TcpStream::connect(address)
        .await
        .map_err(|_| Error::InvalidAddress)?;

    let join = ClientMessage::JoinRequest {
        username: username.into(),
    };
    write_msg(&mut socket, &join)
        .await
        .map_err(|_| Error::ServerError)?;

    match read_msg::<_, ServerMessage>(&mut socket)
        .await
        .map_err(|_| Error::ServerError)?
    {
        ServerMessage::JoinAccepted {
            history,
            participants,
        } => {
            tx.send(NetworkMessage::ServerMessage(ServerMessage::JoinAccepted {
                history,
                participants,
            }))
            .map_err(|_| Error::ChannelClosed)?;

            Ok(socket)
        }
        _ => Err(Error::ServerError),
    }
}

async fn read_from_ui(
    mut rx: UnboundedReceiver<UiMessage>,
    mut writer: OwnedWriteHalf,
) -> Result<()> {
    while let Some(message) = rx.recv().await {
        match message {
            UiMessage::JoinRoom {
                address: _,
                username: _,
            } => return Err(Error::ProtocolError),
            UiMessage::SendChat { text } => {
                let chat = ClientMessage::Chat { text: text.into() };
                write_msg(&mut writer, &chat)
                    .await
                    .map_err(|_| Error::ServerError)?;
            }
        }
    }

    Ok(())
}

async fn write_to_ui(tx: UnboundedSender<NetworkMessage>, mut reader: OwnedReadHalf) -> Result<()> {
    loop {
        match read_msg::<_, ServerMessage>(&mut reader)
            .await
            .map_err(|_| Error::ServerError)?
        {
            ServerMessage::JoinAccepted {
                history: _,
                participants: _,
            } => return Err(Error::ServerError),
            server_message => tx
                .send(NetworkMessage::ServerMessage(server_message))
                .map_err(|_| Error::ChannelClosed)?,
        }
    }
}
