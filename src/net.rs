
use std::io::{Read, Write};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MsgError {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    SerdeBinary(#[from] bincode::Error)
}

pub fn read_message<Message: for<'m> serde::Deserialize<'m>>(reader: &mut dyn Read) -> Result<Message, MsgError> {
    let mut message_size_buffer = [0; 8];
    reader.read_exact(&mut message_size_buffer)?;

    let message_size = usize::from_be_bytes(message_size_buffer);
    let mut message_body_buffer = vec![0; message_size];
    reader.read_exact(&mut message_body_buffer)?;
    Ok(bincode::deserialize(&message_body_buffer)?)
}

pub fn write_message<Message: serde::Serialize>(writer: &mut dyn Write, message: Message) -> Result<(), MsgError> {
    let message_body_buffer = bincode::serialize(&message)?;
    let message_size_buffer = usize::to_be_bytes(message_body_buffer.len());
    writer.write_all(&message_size_buffer)?;
    writer.write_all(&message_body_buffer)?;
    writer.flush()?;
    Ok(())
}