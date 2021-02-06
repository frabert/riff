use std::{cell::RefCell, io::BufReader};
use std::{fmt::Debug, fs::File, rc::Rc};
use std::{
    io::{Read, Seek},
    path::Path,
};

use crate::{
    constants::{LIST_ID, RIFF_ID, SEQT_ID},
    error::RiffResult,
    FourCC,
};

type RcReader = std::rc::Rc<RefCell<BufReader<std::fs::File>>>;

/// Represents the possible data contained in a `ChunkDisk`.
#[derive(Debug)]
pub enum ChunkDiskType {
    RawData(ChunkDisk),
    Children(ChunkDisk),
    ChildrenNoType(ChunkDisk),
}

impl ChunkDiskType {
    pub fn from_chunk_disk(mut chunk: ChunkDisk) -> RiffResult<ChunkDiskType> {
        let chunk_id = chunk.id()?;
        let result = match chunk_id.as_bytes() {
            RIFF_ID | LIST_ID => ChunkDiskType::Children(chunk),
            SEQT_ID => ChunkDiskType::ChildrenNoType(chunk),
            _ => ChunkDiskType::RawData(chunk),
        };
        Ok(result)
    }
}

/// `ChunkDisk` is an opaque type. The only way to access its content is by converting it into
/// a `ChunkDiskContent`.

/// Represents a lazy reader of a chunk in a RIFF file.
#[derive(Debug)]
pub struct ChunkDisk {
    pos: u32,
    reader: RcReader,
}

impl ChunkDisk {
    pub fn id(&mut self) -> RiffResult<FourCC> {
        let id = self.read_4_bytes_from_offset(0)?;
        let result = FourCC::new(&id);
        Ok(result)
    }

    pub fn payload_len(&mut self) -> RiffResult<u32> {
        let id = self.read_4_bytes_from_offset(4)?;
        let result = u32::from_le_bytes(id);
        Ok(result)
    }

    pub fn chunk_type(&mut self) -> RiffResult<FourCC> {
        let id = self.read_4_bytes_from_offset(8)?;
        let result = FourCC::new(&id);
        Ok(result)
    }

    fn from_reader(reader: &RcReader, offset: u32) -> ChunkDisk {
        ChunkDisk {
            pos: offset,
            reader: reader.clone(),
        }
    }

    pub fn from_path<P>(path: P) -> RiffResult<ChunkDisk>
    where
        P: AsRef<Path>,
    {
        let reader = Rc::new(RefCell::new(BufReader::new(File::open(&path)?)));
        Ok(ChunkDisk { pos: 0, reader })
    }

    fn read_4_bytes_from_offset(&mut self, offset: u32) -> RiffResult<[u8; 4]> {
        let mut buffer = [0, 0, 0, 0];
        let pos = (self.pos + offset) as u64;
        let mut reader = self.reader.borrow_mut();
        reader.seek(std::io::SeekFrom::Start(pos))?;
        reader.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    pub fn get_raw_child(&mut self) -> RiffResult<Vec<u8>> {
        let pos = self.pos as u64;
        let payload_len = self.payload_len()? as usize;
        let offset = self.offset_into_data()? as u64;
        let mut result = vec![0; payload_len];
        let mut reader = self.reader.borrow_mut();
        reader.seek(std::io::SeekFrom::Start(pos + offset))?;
        reader.read_exact(&mut result)?;
        Ok(result)
    }

    fn offset_into_data(&mut self) -> RiffResult<usize> {
        Ok(match self.id()?.as_bytes() {
            RIFF_ID | LIST_ID => 12,
            _ => 8,
        })
    }

    pub fn iter(&mut self) -> RiffResult<ChunkDiskIter> {
        let result = match self.id()?.as_bytes() {
            LIST_ID | RIFF_ID => ChunkDiskIter {
                cursor: self.pos + 12,
                cursor_end: self.pos + 12 + self.payload_len()? - 4,
                reader: self.reader.clone(),
                error_occurred: false,
            },
            _ => ChunkDiskIter {
                cursor: self.pos + 8,
                cursor_end: self.pos + 8 + self.payload_len()?,
                reader: self.reader.clone(),
                error_occurred: false,
            },
        };
        Ok(result)
    }

    pub fn get_reader(&self) -> RcReader {
        self.reader.clone()
    }
}

#[derive(Debug)]
pub struct ChunkDiskIter {
    cursor: u32,
    cursor_end: u32,
    reader: RcReader,
    error_occurred: bool,
}

impl Iterator for ChunkDiskIter {
    type Item = RiffResult<ChunkDisk>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.error_occurred || self.cursor >= self.cursor_end {
            None
        } else {
            let mut chunk = ChunkDisk::from_reader(&self.reader, self.cursor);
            let payload = chunk.payload_len();
            let result = match payload {
                Ok(len) => {
                    let chunk_size = 8 + len + (len % 2);
                    self.cursor = self.cursor + chunk_size;
                    Some(Ok(chunk))
                }
                Err(err) => {
                    self.error_occurred = true;
                    Some(Err(err))
                }
            };
            result
        }
    }
}
