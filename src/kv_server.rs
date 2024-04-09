use std::{fs, io, thread};
use std::env::current_dir;
use std::error::Error;
use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use slog::{Logger};
use crate::engine::{KvsEngine, EngineType, Sled};
use crate::{error, info, KvError};
use crate::kv::KvStore;
use crate::message::{Request, Response};
use crate::net::{MsgError, read_message, write_message};

pub struct KvServer {
    logger: Logger,
    engine: Arc<Mutex<Box<dyn KvsEngine + Send>>>,
}

impl KvServer {
    pub fn new(logger: Logger, engine_type: EngineType) -> Result<KvServer, Box<dyn Error>> {
        Ok(KvServer {
            logger,
            engine: Arc::new(Mutex::new(new_engine(engine_type)?)),
        })
    }

    pub fn start<Addr: ToSocketAddrs>(&mut self, address: Addr) -> io::Result<()> {
        let listener = TcpListener::bind(address)?;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    info!(self.logger, "accept connection from {:?}", stream.peer_addr());

                    let logger = self.logger.clone();
                    let engine = Arc::clone(&self.engine);
                    thread::spawn(move || {
                        Self::handle_connection(&logger, engine, stream)
                    });
                }
                Err(err) => {
                    error!(self.logger, "{}", err);
                }
            }
        }

        Ok(())
    }

    fn handle_connection(logger: &Logger, engine: Arc<Mutex<Box<dyn KvsEngine + Send>>>, stream: TcpStream) {
        let mut reader = BufReader::new(&stream);
        let mut writer = BufWriter::new(&stream);
        loop {
            match read_message::<Request>(&mut reader) {
                Ok(request) => {
                    let _ = Self::handle_request(logger, Arc::clone(&engine), &mut writer, request)
                        .inspect_err(|e| {
                            error!(logger, "{}", e)
                        });
                }
                Err(err) => {
                    match err {
                        MsgError::Io(ref err) => {
                            match err.kind() {
                                io::ErrorKind::UnexpectedEof => {
                                    info!(logger, "client disconnect {:?}", stream.peer_addr());
                                    return;
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }

                    error!(logger, "{}", err);
                }
            }
        }
    }

    fn handle_request(
        logger: &Logger,
        engine: Arc<Mutex<Box<dyn KvsEngine + Send>>>,
        writer: &mut dyn Write,
        request: Request) -> Result<(), Box<dyn Error>> {
        match request {
            Request::Get { key } => {
                match engine.lock().unwrap().get(key.clone()) {
                    Ok(value) => {
                        info!(logger, "get {} {:?}", key, value);
                        let res = Response::OkValue {
                            value
                        };
                        write_message::<Response>(writer, res)?;
                    }
                    Err(e) => {
                        error!(logger, "get {} {}", key, e);
                        write_message::<Response>(writer, Response::ErrorUnknown {
                            message: e.to_string()
                        })?;
                    }
                }
            }
            Request::Set { key, value } => {
                match engine.lock().unwrap().set(key.clone(), value.clone()) {
                    Ok(_) => {
                        info!(logger, "set {} {}", key, value);
                        write_message::<Response>(writer, Response::OkNoContent)?;
                    }
                    Err(e) => {
                        error!(logger, "set {} {} {}", key, value, e);
                        write_message::<Response>(writer, Response::ErrorUnknown {
                            message: e.to_string()
                        })?;
                    }
                }
                write_message::<Response>(writer, Response::OkNoContent)?;
            }
            Request::Remove { key } => {
                match engine.lock().unwrap().remove(key.clone()) {
                    Ok(_) => {
                        info!(logger, "rm {}", key);
                        write_message::<Response>(writer, Response::OkNoContent)?;
                    }
                    Err(e) => {
                        error!(logger, "rm {} {}", key, e);
                        match e {
                            KvError::KeyNotFound => {
                                write_message::<Response>(writer, Response::ErrorKeyNotFound)?;
                            }
                            _ => {
                                write_message::<Response>(writer, Response::ErrorUnknown {
                                    message: e.to_string()
                                })?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

fn new_engine(engine_type: EngineType) -> Result<Box<dyn KvsEngine + Send>, Box<dyn Error>> {
    let dir = current_dir()?;
    let curr_engine = current_engine(dir.clone())?;

    match engine_type {
        EngineType::KvStore => {
            if let Some(curr_engine) = curr_engine {
                match curr_engine {
                    EngineType::Sled => {
                        return Err(Box::from("engine mismatch"))
                    }
                    _ => {}
                }
            }

            set_engine(dir.clone(), engine_type)?;
            Ok(Box::new(KvStore::open(dir)?))
        }
        EngineType::Sled => {
            if let Some(curr_engine) = curr_engine {
                match curr_engine {
                    EngineType::KvStore => {
                        return Err(Box::from("engine mismatch"))
                    }
                    _ => {}
                }
            }

            set_engine(dir.clone(), engine_type)?;
            Ok(Box::new(Sled::open(dir)?))
        }
        EngineType::Auto => {
            match current_engine(dir)? {
                Some(engine_type) => {
                    new_engine(engine_type)
                }
                None => new_engine(EngineType::KvStore)
            }
        }
    }
}

fn current_engine(dir: PathBuf) -> Result<Option<EngineType>, Box<dyn Error>> {
    let engine_cfg = dir.join("engine");
    if !engine_cfg.exists() {
        return Ok(None)
    }

    Ok(Some(fs::read_to_string(engine_cfg)?.parse()?))
}

fn set_engine(dir: PathBuf, engine_type: EngineType) -> Result<(), Box<dyn Error>> {
    let engine_cfg = dir.join("engine");
    Ok(fs::write(engine_cfg, format!("{}", engine_type))?)
}