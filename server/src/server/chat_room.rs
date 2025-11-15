use std::{collections::HashMap, sync::Arc};

use common::{
    protocol::{ChatMessage, ServerMessage, encode_message},
    uuid::Uid,
};
use tokio::sync::{Mutex, RwLock};

use crate::{
    error::{Error, Result},
    server::participant::Participant,
};

pub struct ChatRoom {
    participants: RwLock<HashMap<Uid, Participant>>,
    history: Mutex<Vec<Arc<ChatMessage>>>,
}

impl ChatRoom {
    pub fn new() -> Self {
        Self {
            participants: RwLock::new(HashMap::new()),
            history: Mutex::new(Vec::new()),
        }
    }

    pub async fn join(&self, uuid: &Uid, participant: Participant) -> Result<()> {
        self.add_participant(uuid.clone(), participant.clone())
            .await;

        let join_accepted = ServerMessage::JoinAccepted {
            history: self.get_history().await,
            participants: self.get_usernames().await,
        };
        let bytes = encode_message(&join_accepted)
            .await
            .map_err(|_| Error::EncodeError {
                message: join_accepted,
            })?;
        let _ = participant.tx.send(bytes);

        let message = ServerMessage::UserJoined {
            uuid: uuid.clone(),
            username: participant.username.clone(),
        };
        self.broadcast(message.into(), uuid).await
    }

    pub async fn leave(&self, uuid: &Uid) -> Result<()> {
        self.remove_participant(uuid).await;

        let message = ServerMessage::UserLeft { uuid: uuid.clone() };
        self.broadcast(message, uuid).await
    }

    pub async fn relay_message(&self, message: ChatMessage, sender: &Uid) -> Result<()> {
        let message = Arc::new(message);
        self.add_history(message.clone()).await;

        self.broadcast(ServerMessage::Chat(message), sender).await
    }

    pub async fn get_history(&self) -> Vec<Arc<ChatMessage>> {
        self.history.lock().await.clone()
    }

    pub async fn get_usernames(&self) -> Vec<(Uid, Arc<str>)> {
        self.participants
            .read()
            .await
            .iter()
            .map(|(uuid, participant)| (uuid.clone(), participant.username.clone()))
            .collect()
    }

    async fn add_participant(&self, uuid: Uid, participant: Participant) {
        self.participants.write().await.insert(uuid, participant);
    }

    async fn remove_participant(&self, uuid: &Uid) {
        self.participants.write().await.remove(uuid);
    }

    async fn add_history(&self, message: Arc<ChatMessage>) {
        self.history.lock().await.push(message);
    }

    async fn broadcast(&self, message: ServerMessage, sender: &Uid) -> Result<()> {
        let bytes = encode_message(&message)
            .await
            .map_err(|_| Error::EncodeError { message })?;

        let participants = {
            let participants = self.participants.read().await;
            participants
                .iter()
                .filter(|(username, _)| *username != sender)
                .map(|(_, participant)| participant.clone())
                .collect::<Vec<_>>()
        };

        for participant in participants {
            let _ = participant.tx.send(bytes.clone());
        }

        Ok(())
    }
}
