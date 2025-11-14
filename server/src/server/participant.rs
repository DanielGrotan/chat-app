use std::sync::Arc;

use bytes::Bytes;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct Participant {
    pub username: Arc<str>,
    pub tx: mpsc::UnboundedSender<Bytes>,
}

impl Participant {
    pub fn new(username: impl Into<Arc<str>>, tx: mpsc::UnboundedSender<Bytes>) -> Self {
        Participant {
            username: username.into(),
            tx,
        }
    }
}
