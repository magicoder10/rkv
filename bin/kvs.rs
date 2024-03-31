use clap::{arg, command, Command};
use kvs::{KvError, KvStore};
use std::env;
use std::env::current_dir;
use std::process::exit;
use std::string::String;

fn main() -> kvs::Result<()> {
    let matches = command!()
        .name(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(Command::new("get").arg(arg!([key]).required(true)))
        .subcommand(
            Command::new("set")
                .arg(arg!([key]).required(true))
                .arg(arg!([value]).required(true)),
        )
        .subcommand(Command::new("rm").arg(arg!([key])))
        .get_matches();
    match matches.subcommand() {
        Some(("get", sub_matches)) => {
            let key = sub_matches.get_one::<String>("key").unwrap().to_string();
            let mut store = KvStore::open(current_dir()?)?;
            match store.get(key)? {
                Some(value) => {
                    println!("{}", value)
                }
                _ => println!("Key not found")
            }
        }
        Some(("set", sub_matches)) => {
            let key = sub_matches.get_one::<String>("key").unwrap().to_string();
            let value = sub_matches.get_one::<String>("value").unwrap().to_string();
            let mut store = KvStore::open(current_dir()?)?;
            store.set(key, value)?;
        }
        Some(("rm", sub_matches)) => {
            let key = sub_matches.get_one::<String>("key").unwrap().to_string();
            let mut store = KvStore::open(current_dir()?)?;
            match store.remove(key) {
                Ok(())=>{},
                Err(KvError::KeyNotFound) => {
                    println!("Key not found");
                    exit(1);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}
