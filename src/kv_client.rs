use std::error::Error;
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use crate::err;
use crate::KvError::{KeyNotFound, Unknown};
use crate::message::{Request, Response};
use crate::net::{read_message, write_message};



pub struct KvClient {
    reader: Box<dyn Read>,
    writer: Box<dyn Write>,
}

impl KvClient {
    pub fn connect<Addr: ToSocketAddrs>(addr: Addr) -> Result<Self, Box<dyn Error>> {
        let stream = TcpStream::connect(addr)?;
        let read_stream = stream.try_clone()?;
        let write_stream = stream.try_clone()?;
        Ok(KvClient {
            reader: Box::new(BufReader::new(read_stream)),
            writer: Box::new(BufWriter::new(write_stream)),
        })
    }

    pub fn get(&mut self, key: String) -> err::Result<Option<String>> {
        write_message(&mut self.writer, Request::Get {
            key
        })?;
        let response = read_message::<Response>(&mut self.reader)?;
        match response {
            Response::OkValue{value} => {
                Ok(value)
            }
            Response::OkNoContent => {
                Err(Unknown)
            }
            Response::ErrorKeyNotFound => {
                Err(KeyNotFound)
            }
            Response::ErrorUnknown{..} => {
                Err(Unknown)
            }
        }
    }

    pub fn set(&mut self, key: String, value: String) -> err::Result<()> {
        write_message(&mut self.writer, Request::Set {
            key,
            value
        })?;
        let response = read_message::<Response>(&mut self.reader)?;
        match response {
            Response::OkValue{ .. } => {
                Err(Unknown)
            }
            Response::OkNoContent => {
                Ok(())
            }
            Response::ErrorKeyNotFound => {
                Err(Unknown)
            }
            Response::ErrorUnknown{..} => {
                Err(Unknown)
            }
        }
    }

    pub fn remove(&mut self, key: String) -> err::Result<()> {
        write_message(&mut self.writer, Request::Remove {
            key
        })?;
        let response = read_message::<Response>(&mut self.reader)?;
        match response {
            Response::OkValue{..} => {
                Err(Unknown)
            }
            Response::OkNoContent => {
                Ok(())
            }
            Response::ErrorKeyNotFound => {
                Err(KeyNotFound)
            }
            Response::ErrorUnknown{..} => {
                Err(Unknown)
            }
        }
    }
}