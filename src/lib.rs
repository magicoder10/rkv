extern crate core;

pub use kv::KvStore;
pub use err::{Result, KvError};

mod err;
mod cmd;
mod stream;
mod kv;



