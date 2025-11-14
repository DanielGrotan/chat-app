use std::{fmt::Display, sync::Arc};

use common::{protocol::ServerMessage, uuid::Uid};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ConnectionClosed { uuid: Uid, username: Arc<str> },
    EncodeError { message: ServerMessage },
    FailedToJoin,
    AlreadyJoined { uuid: Uid, username: Arc<str> },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
