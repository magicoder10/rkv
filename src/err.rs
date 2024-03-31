use std::{io, result};
use failure::Fail;

#[derive(Fail, Debug)]
pub enum KvError {
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),

    #[fail(display = "{}", _0)]
    Serde(#[cause] serde_json::Error),

    #[fail(display = "Unexpected command type")]
    UnexpectedCommandType,

    #[fail(display = "Key not found")]
    KeyNotFound,
}

impl From<io::Error> for KvError {
    fn from(error: io::Error) -> Self {
        KvError::Io(error)
    }
}

impl From<serde_json::Error> for KvError {
    fn from(error: serde_json::Error) -> Self {
        KvError::Serde(error)
    }
}

pub type Result<Value> = result::Result<Value, KvError>;