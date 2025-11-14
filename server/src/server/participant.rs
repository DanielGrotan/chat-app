use std::sync::Arc;

use common::protocol::ServerMessage;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct Participant {
    pub username: String,
    pub tx: mpsc::UnboundedSender<Arc<ServerMessage>>,
}

impl Participant {
    pub fn new(username: String, tx: mpsc::UnboundedSender<Arc<ServerMessage>>) -> Self {
        Participant { username, tx }
    }
}
