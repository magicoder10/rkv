use std::net::SocketAddr;
use std::process::exit;
use clap::{arg, command, Command};
use slog::{Drain, Duplicate, Level, Logger, o};
use kvs::{debug, error, KvClient, KvError};

fn main() {
    let stdout_decorator = slog_term::TermDecorator::new()
        .stdout()
        .build();
    let stdout_drain = slog_term::CompactFormat::new(stdout_decorator)
        .build()
        .fuse()
        .filter(|record| record.level() != Level::Error && record.level() != Level::Debug);
    let stderr_decorator = slog_term::TermDecorator::new()
        .stderr()
        .build();
    let stderr_drain = slog_term::CompactFormat::new(stderr_decorator)
        .build()
        .fuse()
        .filter(|record| record.level() == Level::Error);

    let drain = Duplicate::new(stdout_drain, stderr_drain)
        .fuse();
    let async_drain = slog_async::Async::new(drain)
        .build()
        .fuse();
    let logger = Logger::root(async_drain, o!());
    let matches = command!()
        .name(env!("CARGO_BIN_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            arg!(--addr <Value>).global(true),
        )
        .subcommands([
            Command::new("get").arg(
                arg!([key]).required(true)
            ),
            Command::new("set").args([
                arg!(<key>).required(true),
                arg!(<value>).required(true)
            ]),
            Command::new("rm").arg(
                arg!(<key>).required(true)
            )
        ])
        .get_matches();
    let addr: SocketAddr = matches.get_one::<String>("addr")
        .unwrap_or(&"127.0.0.1:4000".to_string())
        .parse()
        .unwrap();

    match matches.subcommand() {
        Some(("get", arg_matches)) => {
            let key = arg_matches.get_one::<String>("key").unwrap();
            debug!(logger, "get {} {}", addr, key);

            let kv_client = KvClient::connect(addr);
            if let Err(e) = kv_client {
                error!(logger, "{:?}", e);
                exit(1);
            }

            match kv_client.unwrap().get(key.to_string()) {
                Ok(value) => {
                    match value {
                        Some(value) => {
                            println!("{}", value)
                        }
                        None => {
                            println!("Key not found");
                        }
                    }
                }
                Err(e) => {
                    error!(logger, "{}", e);
                    exit(1);
                }
            }
        }
        Some(("set", arg_matches)) => {
            let key = arg_matches.get_one::<String>("key").unwrap();
            let value = arg_matches.get_one::<String>("value").unwrap();
            debug!(logger, "set {} {} {}", addr, key, value);

            let kv_client = KvClient::connect(addr);
            if let Err(e) = kv_client {
                error!(logger, "{:?}", e);
                exit(1);
            }

            match kv_client.unwrap().set(key.to_string(), value.to_string()) {
                Err(e) => {
                    error!(logger, "{}", e);
                    exit(1);
                }
                _ => {}
            }
        }
        Some(("rm", arg_matches)) => {
            let key = arg_matches.get_one::<String>("key").unwrap();
            debug!(logger, "rm {} {}", addr, key);

            let kv_client = KvClient::connect(addr);
            if let Err(e) = kv_client {
                error!(logger, "{:?}", e);
                exit(1);
            }

            match kv_client.unwrap().remove(key.to_string()) {
                Err(e) => {
                    match e {
                        KvError::KeyNotFound => {
                            eprintln!("Key not found");
                            exit(1);
                        }
                        _ => {
                            error!(logger, "{}", e);
                            exit(1);
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {
            unreachable!()
        }
    }
}