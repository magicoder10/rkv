use std::{io, result};
use std::str::Utf8Error;
use thiserror::Error;
use crate::net::MsgError;

#[derive(Error, Debug)]
pub enum KvError {
    #[error("{0}")]
    Io(#[from] io::Error),

    #[error("{0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("{0}")]
    SerdeBinary(#[from] bincode::Error),

    #[error("{0}")]
    Sled(#[from] sled::Error),

    #[error("{0}")]
    Encode(#[from] Utf8Error),

    #[error("Unexpected command type")]
    UnexpectedCommandType,

    #[error("Key not found")]
    KeyNotFound,

    #[error("Unknown")]
    Unknown,
}

impl From<MsgError> for KvError {
    fn from(value: MsgError) -> Self {
        match value {
            MsgError::Io(e) => KvError::Io(e),
            MsgError::SerdeBinary(e) => KvError::SerdeBinary(e),
        }
    }
}

pub type Result<Value> = result::Result<Value, KvError>;