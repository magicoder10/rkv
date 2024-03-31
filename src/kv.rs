use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::{fs, io};
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write, Read};
use std::path::{Path, PathBuf};
use serde_json::Deserializer;
use crate::cmd::Command;
use crate::err::KvError;
use crate::err::Result;
use crate::stream::{BufReaderWithPos, BufWriterWithPos};

const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

#[derive(Debug)]
pub struct CommandPosition {
    log_start_pos: u64,
    size: u64,
    gen: u64,
}

pub struct KvStore {
    dir: PathBuf,
    writer: BufWriterWithPos<File>,
    readers: HashMap<u64, BufReaderWithPos<File>>,
    index: BTreeMap<String, CommandPosition>,
    stale_data_size: u64,
    current_gen: u64,
}

impl KvStore {
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command::set(key.clone(), value);
        let log_start_pos = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;

        let log_end_pos = self.writer.pos;
        let size = log_end_pos - log_start_pos;
        let val = self.index.insert(key.clone(), CommandPosition {
            log_start_pos,
            size,
            gen: self.current_gen,
        }.into());
        if let Some(val) = val {
            self.stale_data_size += val.size;
        }

        if self.stale_data_size >= COMPACTION_THRESHOLD {
            self.compact_log()?;
        }

        Ok(())
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.index.get(&key) {
            Some(cmd_pos) => {
                let reader = self.readers
                    .get_mut(&cmd_pos.gen)
                    .ok_or(KvError::KeyNotFound)?;
                reader.seek(SeekFrom::Start(cmd_pos.log_start_pos))?;
                let cmd_reader = reader.take(cmd_pos.size);
                match serde_json::from_reader(cmd_reader)? {
                    Command::Set { value, .. } => {
                        return Ok(Some(value));
                    }
                    _ => {
                        Err(KvError::UnexpectedCommandType)
                    }
                }
            }
            None => Ok(None)
        }
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.index.contains_key(&key) {
            let cmd = Command::remove(key.clone());
            let log_start_pos = self.writer.pos;
            serde_json::to_writer(&mut self.writer, &cmd)?;
            self.writer.flush()?;

            let log_end_pos = self.writer.pos;

            let val = self.index.remove(&key);
            if let Some(old_cmd) = val {
                self.stale_data_size += old_cmd.size;
            }

            let remove_cmd_size = log_end_pos - log_start_pos;
            self.stale_data_size += remove_cmd_size;

            if self.stale_data_size >= COMPACTION_THRESHOLD {
                self.compact_log()?;
            }

            Ok(())
        } else {
            Err(KvError::KeyNotFound)
        }
    }

    pub fn open(dir: impl Into<PathBuf>) -> Result<KvStore> {
        let dir = dir.into();
        fs::create_dir_all(&dir)?;

        let mut index: BTreeMap<String, CommandPosition> = BTreeMap::new();
        let mut readers: HashMap<u64, BufReaderWithPos<File>> = HashMap::new();
        let gens = get_sorted_gens(&dir)?;

        let mut stale_data_size = 0;
        for &gen in &gens {
            let file_path = log_path(&dir, gen);
            let read_file = OpenOptions::new()
                .read(true)
                .open(&file_path)?;
            let mut reader = BufReaderWithPos::new(read_file)?;
            stale_data_size += load(gen, &mut index, &mut reader)?;
            readers.insert(gen, reader);
        }

        let current_gen = gens.last().unwrap_or(&0)+1;

        let writer = new_log_file(&dir, current_gen, &mut readers)?;
        Ok(KvStore {
            dir,
            writer,
            readers,
            index,
            stale_data_size,
            current_gen,
        })
    }

    fn compact_log(&mut self) -> Result<()> {
        let compaction_gen = self.current_gen+1;
        self.current_gen += 2;

        self.writer = new_log_file(&self.dir, self.current_gen, &mut self.readers)?;
        let mut compaction_writer = new_log_file(&self.dir, compaction_gen, &mut self.readers)?;

        let mut write_pos = 0;
        for cmd_pos in self.index.values_mut() {
            let reader = self.readers.get_mut(&cmd_pos.gen)
                .ok_or(KvError::KeyNotFound)?;
            if reader.pos != cmd_pos.log_start_pos {
                reader.seek(SeekFrom::Start(cmd_pos.log_start_pos))?;
            }

            let mut cmd_reader = reader.take(cmd_pos.size);
            let len = io::copy(&mut cmd_reader, &mut compaction_writer)?;
            *cmd_pos = CommandPosition{
                log_start_pos: write_pos,
                size: len,
                gen: compaction_gen
            };
            write_pos += len;
        }

        compaction_writer.flush()?;

        let stale_gens: Vec<_> = self.readers
            .keys()
            .filter(|&&gen| gen < compaction_gen)
            .cloned()
            .collect();

        for gen in stale_gens {
            self.readers.remove(&gen);
            fs::remove_file(log_path(&self.dir, gen))?;
        }

        self.stale_data_size = 0;
        Ok(())
    }
}

fn log_path(dir: &Path, gen: u64) -> PathBuf {
    dir.join(format!("{}.log", gen))
}

fn get_sorted_gens(dir: &Path) -> Result<Vec<u64>> {
    let mut gens: Vec<u64> = fs::read_dir(dir)?
        .flat_map(|entry| -> Result<_> { Ok(entry?.path())})
        .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
        .flat_map(|path|path
            .file_name()
            .and_then(OsStr::to_str)
            .map(|s|s.trim_end_matches(".log"))
            .map(str::parse::<u64>)
            )
        .flatten()
        .collect();
    gens.sort_unstable();
    Ok(gens)
}

fn load(gen: u64, index: &mut BTreeMap<String, CommandPosition>, reader: &mut BufReaderWithPos<File>) -> Result<u64> {
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut cmd_stream = Deserializer::from_reader(reader)
        .into_iter::<Command>();
    let mut stale_data_size: u64 = 0;
    while let Some(cmd_res) = cmd_stream.next() {
        let new_pos = cmd_stream.byte_offset() as u64;
        let cmd = cmd_res?;
        match cmd.clone() {
            Command::Set {key, ..} => {
                let size = new_pos - pos;
                let val = index.insert(key, CommandPosition{
                    log_start_pos: pos,
                    size,
                    gen
                });
                if let Some(old_cmd) = val {
                    stale_data_size+= old_cmd.size;
                }
            }
            Command::Remove {key} => {
                let val = index.remove(&key);
                if let Some(old_cmd) = val {
                    stale_data_size+= old_cmd.size;
                }

                let remove_cmd_size: u64 = new_pos - pos;
                stale_data_size += remove_cmd_size;
            }
        }

        pos = new_pos;
    }

    Ok(stale_data_size)
}

fn new_log_file(dir: &PathBuf, gen: u64, readers: &mut HashMap<u64, BufReaderWithPos<File>>)-> Result<BufWriterWithPos<File>> {
        let file_path = log_path(dir, gen);
        let write_file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&file_path)?;

        let read_file = OpenOptions::new()
            .read(true)
            .open(&file_path)?;
        let reader = BufReaderWithPos::new(read_file)?;
        readers.insert(gen, reader);

        return Ok(BufWriterWithPos::new(write_file)?)
}