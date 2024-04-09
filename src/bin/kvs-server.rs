use std::net::SocketAddr;
use std::process::exit;
use clap::{arg, command};
use slog::{Drain, Duplicate, Level, Logger, o};
use kvs::{error, info, KvServer};

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
        .args(
            [
                arg!(--addr <Value>).global(true),
                arg!(--engine <Value>).global(true),
            ]
        )
        .get_matches();
    let addr: SocketAddr = matches.get_one::<String>("addr")
        .unwrap_or(&"127.0.0.1:4000".to_string())
        .parse()
        .unwrap();

    let engine_type_str = matches.get_one::<String>("engine");
    info!(logger, "version: {}, address: {}, engine: {:?}", env!("CARGO_PKG_VERSION"), addr, engine_type_str);

    let engine_type = engine_type_str.unwrap_or(&"auto".to_string()).parse();
    if let Err(e) = engine_type {
        error!(logger, "{:?}", e);
        exit(1);
    }

    let kv_server = KvServer::new(logger.clone(), engine_type.unwrap());
    if let Err(e) = kv_server {
        error!(logger, "{}", e);
        exit(1);
    }

    if let Err(e) = kv_server.unwrap().start(addr.clone()) {
        error!(logger, "{}", e);
        exit(1);
    }
}