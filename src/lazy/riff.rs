use std::convert::TryFrom;
use std::fmt::Debug;
use std::io::{Read, Seek};
use std::path::PathBuf;

use crate::{
    constants::{LIST_ID, RIFF_ID, SEQT_ID},
    error::{RiffError, RiffResult},
};

/// Lazy version of `ChunkId`.
#[derive(Debug, Clone)]
pub struct ChunkId {
    data: [u8; 4],
}

impl ChunkId {
    pub fn as_str(&self) -> Result<&str, RiffError> {
        // TODO: Handle this error.
        Ok(std::str::from_utf8(&self.data)?)
    }
}

/// Lazy version of `ChunkType`.
#[derive(Debug, Clone)]
pub struct ChunkType {
    data: [u8; 4],
}

impl ChunkType {
    pub fn as_str(&self) -> RiffResult<&str> {
        // TODO: Handle this error.
        Ok(std::str::from_utf8(&self.data)?)
    }
}

#[derive(Debug)]
pub enum ChunkContents {
    RawData(ChunkId, Vec<u8>),
    Children(ChunkId, ChunkType, Vec<ChunkContents>),
    ChildrenNoType(ChunkId, Vec<ChunkContents>),
}

impl TryFrom<Chunk> for ChunkContents {
    type Error = RiffError;

    fn try_from(chunk: Chunk) -> RiffResult<Self> {
        let chunk_id = chunk.id().clone();
        match chunk_id.as_str()? {
            RIFF_ID | LIST_ID => {
                let chunk_type = chunk.chunk_type();
                let child_contents = chunk
                    .iter()?
                    .map(|child| ChunkContents::try_from(child?))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ChunkContents::Children(
                    chunk_id,
                    chunk_type.clone()?,
                    child_contents,
                ))
            }
            SEQT_ID => {
                let child_contents = chunk
                    .iter()?
                    .map(|child| ChunkContents::try_from(child?))
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

    fn from_path(path: PathBuf, pos: u32) -> RiffResult<Chunk> {
        let pos = pos as u64;
        let mut inner_reader = std::fs::File::open(&path)?;
        let id_buff = Chunk::read_4_bytes(&mut inner_reader, pos)?;
        let payload_len_buff = Chunk::read_4_bytes(&mut inner_reader, pos + 4)?;
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

    fn read_4_bytes<R>(reader: &mut R, pos: u64) -> RiffResult<[u8; 4]>
    where
        R: Read + Seek,
    {
        let mut buffer: [u8; 4] = [0; 4];
        reader.seek(std::io::SeekFrom::Start(pos))?;
        reader.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    pub fn get_raw_child(&self) -> RiffResult<Vec<u8>> {
        let pos = self.pos as u64;
        let payload_len = self.payload_len as usize;
        let offset = self.offset_into_data()? as u64;
        let mut result = vec![0; payload_len];
        let mut reader = std::fs::File::open(&self.path)?;
        reader.seek(std::io::SeekFrom::Start(pos + offset))?;
        reader.read_exact(&mut result)?;
        Ok(result)
    }

    fn offset_into_data(&self) -> RiffResult<usize> {
        Ok(match self.id().as_str()? {
            RIFF_ID | LIST_ID => 12,
            _ => 8,
        })
    }

    pub fn iter(&self) -> RiffResult<ChunkIter> {
        Ok(match self.id().as_str()? {
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
        })
    }
}

#[derive(Debug)]
pub struct ChunkIter {
    cursor: u32,
    cursor_end: u32,
    path: PathBuf,
}

impl Iterator for ChunkIter {
    type Item = RiffResult<Chunk>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.cursor_end {
            None
        } else {
            match Chunk::from_path(self.path.clone(), self.cursor) {
                Ok(chunk) => {
                    self.cursor = self.cursor + 8 + chunk.payload_len + (chunk.payload_len % 2);
                    Some(Ok(chunk))
                }
                // TODO: How to propagate this kind of error?
                // We need to correctly parse chunks to proceed this iterator, but we obviously failed
                // So we would like to stop...but also give the user that something wrong happened,
                // and its not just the end of file.
                Err(_) => None,
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Riff {
    path: PathBuf,
}

impl TryFrom<Riff> for Chunk {
    type Error = RiffError;

    fn try_from(value: Riff) -> Result<Self, Self::Error> {
        Chunk::from_path(value.path, 0)
    }
}

#[allow(dead_code)]
impl Riff {
    pub fn from_path<T>(path: T) -> RiffResult<Self>
    where
        T: Into<PathBuf>,
    {
        let path = path.into();
        let mut reader = std::fs::File::open(&path)?;
        let mut buffer = [0; 4];
        reader.seek(std::io::SeekFrom::Start(0))?;
        reader.read_exact(&mut buffer)?;
        let str = std::str::from_utf8(&buffer)?;
        match str {
            RIFF_ID => Ok(Riff { path }),
            _ => Err(RiffError::NoneError),
        }
    }
}
