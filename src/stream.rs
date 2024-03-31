use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use crate::err::Result;

pub struct BufWriterWithPos<Writable: Write + Seek> {
    writer: BufWriter<Writable>,
    pub pos: u64
}

impl<Writable: Write + Seek> BufWriterWithPos<Writable> {
    pub fn new(mut writable: Writable) -> Result<Self> {
        let pos = writable.seek(SeekFrom::End(0))?;
        Ok(BufWriterWithPos{
            writer: BufWriter::new(writable),
            pos,
        })
    }
}

impl<Writable: Write + Seek> Write for BufWriterWithPos<Writable> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl<Writable: Write + Seek> Seek for BufWriterWithPos<Writable> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}

pub struct BufReaderWithPos<Readable: Read + Seek> {
    reader: BufReader<Readable>,
    pub pos: u64
}

impl<Readable: Read + Seek> BufReaderWithPos<Readable> {
    pub fn new(mut readable: Readable) -> Result<Self> {
        let pos = readable.seek(SeekFrom::Start(0))?;
        Ok(BufReaderWithPos{
            reader: BufReader::new(readable),
            pos,
        })
    }
}

impl<Readable: Read + Seek> Read for BufReaderWithPos<Readable> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
}

impl<Readable: Read + Seek> Seek for BufReaderWithPos<Readable> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}
