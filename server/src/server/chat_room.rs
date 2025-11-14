use std::{collections::HashMap, sync::Arc};

use common::{
    protocol::{ChatMessage, ServerMessage},
    uuid::Uid,
};
use tokio::sync::Mutex;

use crate::server::participant::Participant;

pub struct ChatRoom {
    participants: Mutex<HashMap<Uid, Participant>>,
    history: Mutex<Vec<ChatMessage>>,
}

impl ChatRoom {
    pub fn new() -> Self {
        Self {
            participants: Mutex::new(HashMap::new()),
            history: Mutex::new(Vec::new()),
        }
    }

    pub async fn join(&self, uuid: &Uid, participant: Participant) {
        let message = ServerMessage::UserJoined {
            uuid: uuid.clone(),
            username: participant.username.clone(),
        };
        self.broadcast(message.into(), uuid).await;

        let mut participants = self.participants.lock().await;
        participants.insert(uuid.clone(), participant);
    }

    pub async fn leave(&self, uuid: &Uid) {
        let message = ServerMessage::UserLeft { uuid: uuid.clone() };
        self.broadcast(message.into(), uuid).await;

        let mut participants = self.participants.lock().await;
        participants.remove(uuid);
    }

    pub async fn relay_message(&self, message: ChatMessage, sender: &Uid) {
        self.add_history(message.clone()).await;
        self.broadcast(Arc::new(ServerMessage::Chat(message)), sender)
            .await;
    }

    pub async fn get_history(&self) -> Vec<ChatMessage> {
        self.history.lock().await.clone()
    }

    pub async fn get_usernames(&self) -> Vec<(Uid, String)> {
        self.participants
            .lock()
            .await
            .iter()
            .map(|(uuid, participant)| (uuid.clone(), participant.username.clone()))
            .collect()
    }

    async fn add_history(&self, message: ChatMessage) {
        self.history.lock().await.push(message);
    }

    async fn broadcast(&self, message: Arc<ServerMessage>, sender: &Uid) {
        let participants = {
            let participants = self.participants.lock().await;
            participants
                .iter()
                .filter(|(username, _)| *username != sender)
                .map(|(_, participant)| participant.clone())
                .collect::<Vec<_>>()
        };

        for participant in participants {
            let _ = participant.tx.send(message.clone());
        }
    }
}
