use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::{Path};
use std::str::FromStr;
use thiserror::Error;
use crate::err::{Result};
use crate::KvError;

pub enum EngineType {
    Auto,
    KvStore,
    Sled
}

#[derive(Error, Debug)]
pub struct ParseEngineTypeError;

impl Display for ParseEngineTypeError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "invalid engine type")
    }
}

impl FromStr for EngineType {
    type Err = ParseEngineTypeError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "auto" => Ok(EngineType::Auto),
            "kvs" => Ok(EngineType::KvStore),
            "sled" => Ok(EngineType::Sled),
            _ => Err(ParseEngineTypeError)
        }
    }
}

impl Display for EngineType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineType::Auto => write!(f, "auto"),
            EngineType::KvStore => write!(f, "kvs"),
            EngineType::Sled => write!(f, "sled"),
        }
    }
}

pub trait KvsEngine {
    fn set(&mut self, key: String, value: String) -> Result<()>;
    fn get(&mut self, key: String) -> Result<Option<String>>;
    fn remove(&mut self, key: String) -> Result<()>;
}

pub struct Sled {
    db: sled::Db
}

impl Sled {
    pub fn open(dir: impl AsRef<Path>) -> Result<Sled> {
        let db = sled::open(dir)?;
        Ok(Sled{
            db
        })
    }
}

impl KvsEngine for Sled {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let _ = self.db.insert(key, value.as_bytes())?;
        self.db.flush()?;
        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.db.get(key)? {
            Some(value_vec) => {
                Ok(Some(std::str::from_utf8(&value_vec)?.to_string()))
            }
            None => Ok(None)
        }
    }

    fn remove(&mut self, key: String) -> Result<()> {
        let value =  self.db.remove(key)?;
        match value {
            None => {
                return Err(KvError::KeyNotFound);
            },
            _ => {}
        };

        self.db.flush()?;
        Ok(())
    }
}