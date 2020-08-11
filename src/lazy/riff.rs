use std::convert::TryFrom;
use std::fmt::Debug;
use std::io::{Read, Seek};
use std::path::PathBuf;

use crate::constants::{LIST_ID, RIFF_ID, SEQT_ID};

/// Lazy version of `ChunkId`.
#[derive(Debug, Clone)]
pub struct ChunkId {
    data: [u8; 4],
}

impl ChunkId {
    pub fn as_str(&self) -> &str {
        // TODO: Handle this error.
        std::str::from_utf8(&self.data).unwrap()
    }
}

/// Lazy version of `ChunkType`.
#[derive(Debug, Clone)]
pub struct ChunkType {
    data: [u8; 4],
}

impl ChunkType {
    pub fn as_str(&self) -> &str {
        // TODO: Handle this error.
        std::str::from_utf8(&self.data).unwrap()
    }
}

#[derive(Debug)]
pub enum ChunkContents {
    RawData(ChunkId, Vec<u8>),
    Children(ChunkId, ChunkType, Vec<ChunkContents>),
    ChildrenNoType(ChunkId, Vec<ChunkContents>),
}

impl TryFrom<Chunk> for ChunkContents {
    type Error = std::io::Error;

    fn try_from(chunk: Chunk) -> Result<Self, std::io::Error> {
        let chunk_id = chunk.id().clone();
        match chunk_id.as_str() {
            RIFF_ID | LIST_ID => {
                let chunk_type = chunk.chunk_type();
                let child_contents = chunk
                    .iter()
                    .map(|child| ChunkContents::try_from(child))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ChunkContents::Children(
                    chunk_id,
                    chunk_type.clone().unwrap(),
                    child_contents,
                ))
            }
            SEQT_ID => {
                let child_contents = chunk
                    .iter()
                    .map(|child| ChunkContents::try_from(child))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ChunkContents::ChildrenNoType(chunk_id, child_contents))
            }
            _ => {
                let contents = chunk.get_raw_child()?;
                Ok(ChunkContents::RawData(chunk_id.clone(), contents))
            }
        }
    }
}

#[derive(Debug)]
pub struct Chunk {
    id: ChunkId,
    chunk_type: Option<ChunkType>,
    pos: u32,
    payload_len: u32,
    path: PathBuf,
}

impl Chunk {
    pub fn id(&self) -> &ChunkId {
        &self.id
    }

    pub fn payload_len(&self) -> u32 {
        self.payload_len
    }

    pub fn chunk_type(&self) -> &Option<ChunkType> {
        &self.chunk_type
    }

    fn from_path(path: PathBuf, pos: u32) -> std::io::Result<Chunk> {
        let pos = pos as u64;
        let mut inner_reader = std::fs::File::open(&path).unwrap();
        let id_buff = Chunk::read_4_bytes(&mut inner_reader, pos).unwrap();
        let payload_len_buff = Chunk::read_4_bytes(&mut inner_reader, pos + 4).unwrap();
        let chunk_type = match Chunk::read_4_bytes(&mut inner_reader, pos + 8) {
            Ok(result) => Some(ChunkType { data: result }),
            Err(_) => None,
        };
        Ok(Chunk {
            id: ChunkId { data: id_buff },
            chunk_type,
            pos: pos as u32,
            payload_len: u32::from_le_bytes(payload_len_buff),
            path,
        })
    }

    fn read_4_bytes<R>(reader: &mut R, pos: u64) -> std::io::Result<[u8; 4]>
    where
        R: Read + Seek,
    {
        let mut buffer: [u8; 4] = [0; 4];
        reader.seek(std::io::SeekFrom::Start(pos))?;
        reader.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    pub fn get_raw_child(&self) -> std::io::Result<Vec<u8>> {
        let pos = self.pos as u64;
        let payload_len = self.payload_len as usize;
        let offset = self.offset_into_data() as u64;
        let mut result = vec![0; payload_len];
        let mut reader = std::fs::File::open(&self.path).unwrap();
        reader.seek(std::io::SeekFrom::Start(pos + offset)).unwrap();
        reader.read_exact(&mut result).unwrap();
        Ok(result)
    }

    fn offset_into_data(&self) -> usize {
        match self.id().as_str() {
            RIFF_ID | LIST_ID => 12,
            _ => 8,
        }
    }

    pub fn iter(&self) -> ChunkIter {
        match self.id().as_str() {
            LIST_ID | RIFF_ID => ChunkIter {
                cursor: self.pos + 12,
                cursor_end: self.pos + 12 + self.payload_len - 4,
                path: self.path.clone(),
            },
            _ => ChunkIter {
                cursor: self.pos + 8,
                cursor_end: self.pos + 8 + self.payload_len,
                path: self.path.clone(),
            },
        }
    }
}

#[derive(Debug)]
pub struct ChunkIter {
    cursor: u32,
    cursor_end: u32,
    path: PathBuf,
}

impl Iterator for ChunkIter {
    type Item = Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.cursor_end {
            None
        } else {
            let chunk = Chunk::from_path(self.path.clone(), self.cursor).unwrap();
            self.cursor = self.cursor + 8 + chunk.payload_len + (chunk.payload_len % 2);
            Some(chunk)
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Riff {
    path: PathBuf,
}

impl TryFrom<Riff> for Chunk {
    type Error = std::io::Error;

    fn try_from(value: Riff) -> Result<Self, Self::Error> {
        Chunk::from_path(value.path, 0)
    }
}

#[allow(dead_code)]
impl Riff {
    pub fn from_path<T>(path: T) -> std::io::Result<Self>
    where
        T: Into<PathBuf>,
    {
        let path = path.into();
        Ok(Riff { path })
    }
}
