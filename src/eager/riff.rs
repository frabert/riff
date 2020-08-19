use crate::error::{ChunkTooSmall, ChunkTooSmallForChunkType, PayloadLenMismatch, RiffError};
use crate::{
    constants::{LIST_ID, RIFF_ID, SEQT_ID},
    error::RiffResult,
};
use std::convert::TryFrom;

/// An eager representation of a RIFF file.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Riff {
    data: Vec<u8>,
}

impl<'a> TryFrom<&'a Riff> for Chunk<'a> {
    type Error = RiffError;

    fn try_from(value: &'a Riff) -> RiffResult<Self> {
        Ok(Chunk::from_raw_u8(&value.data, 0)?)
    }
}

#[allow(dead_code)]
impl Riff {
    pub fn id(&self) -> ChunkId {
        let mut buff: [u8; 4] = [0; 4];
        buff.copy_from_slice(&self.data[0..4]);
        ChunkId { data: buff }
    }

    pub fn payload_len(&self) -> u32 {
        let mut buff: [u8; 4] = [0; 4];
        buff.copy_from_slice(&self.data[4..8]);
        u32::from_le_bytes(buff)
    }

    pub fn iter(&self) -> RiffResult<ChunkIter> {
        Ok(Chunk::from_raw_u8(self.data.as_slice(), 0)?.iter())
    }

    pub fn from_file(path: std::path::PathBuf) -> RiffResult<Self> {
        let data = std::fs::read(path)?;
        if data.len() >= 8 {
            Ok(Riff { data })
        } else {
            Err(RiffError::ChunkTooSmall(ChunkTooSmall { data, pos: 0 }))
        }
    }
}

/// Represents a chunk inside a RIFF file that have been created eagerly.
/// To create a lazy version, please see the `Chunk` created by `RiffDisk`.
/// Note that this is an opaque type, to obtain its content, one must convert it into a `ChunkContent`.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk<'a> {
    pos: u32,
    payload_len: u32,
    data: &'a [u8],
}

impl<'a> Chunk<'a> {
    pub fn id(&self) -> ChunkId {
        let pos = self.pos as usize;
        let mut buff: [u8; 4] = [0; 4];
        // SAFETY: Any creation of `Chunk` must occur through `Chunk::from_raw_u8`.
        // In there, we should already checked that the `data[pos..].len()` is _at least_ 8 bytes long.
        buff.copy_from_slice(&self.data[pos..pos + 4]);
        ChunkId { data: buff }
    }

    pub fn payload_len(&self) -> u32 {
        self.payload_len
    }

    pub fn from_raw_u8(data: &[u8], pos: u32) -> RiffResult<Chunk> {
        let pos = pos as usize;
        if data.len() >= pos + 8 {
            let mut payload_buff: [u8; 4] = [0; 4];
            payload_buff.copy_from_slice(&data[pos + 4..pos + 8]);
            Ok(Chunk {
                pos: pos as u32,
                payload_len: u32::from_le_bytes(payload_buff),
                data,
            })
        } else {
            Err(RiffError::ChunkTooSmall(ChunkTooSmall {
                data: Vec::from(data),
                pos,
            }))
        }
    }

    pub fn chunk_type(&self) -> RiffResult<ChunkType> {
        let pos = self.pos as usize;
        if self.data.len() >= pos + 12 {
            let mut buff: [u8; 4] = [0; 4];
            buff.copy_from_slice(&self.data[pos + 8..pos + 12]);
            Ok(ChunkType { data: buff })
        } else {
            Err(RiffError::ChunkTooSmallForChunkType(
                ChunkTooSmallForChunkType {
                    data: Vec::from(self.data),
                    pos,
                },
            ))
        }
    }

    pub fn get_raw_child(&self) -> RiffResult<&'a [u8]> {
        let pos = self.pos as usize;
        let payload_len = self.payload_len as usize;
        let offset = match self.id().as_str() {
            Ok(RIFF_ID) | Ok(LIST_ID) => 12,
            _ => 8,
        };
        if self.data.len() >= pos + offset + payload_len {
            Ok(&self.data[pos + offset..pos + offset + payload_len])
        } else {
            Err(RiffError::PayloadLenMismatch(PayloadLenMismatch {
                data: Vec::from(self.data),
                pos,
                offset,
                payload_len,
            }))
        }
    }

    pub fn iter(&self) -> ChunkIter<'a> {
        match self.id().as_str() {
            Ok(RIFF_ID) | Ok(LIST_ID) => ChunkIter {
                cursor: self.pos + 12,
                // We have to subtract because RIFF_ID and LIST_ID contain chunk type that consumes 4 bytes.
                cursor_end: self.pos + 12 + self.payload_len - 4,
                data: self.data,
                error_occurred: false,
            },
            _ => ChunkIter {
                cursor: self.pos + 8,
                cursor_end: self.pos + 8 + self.payload_len,
                data: self.data,
                error_occurred: false,
            },
        }
    }
}

/// An iterator over the children of a `Chunk`.
#[derive(Debug)]
pub struct ChunkIter<'a> {
    cursor: u32,
    cursor_end: u32,
    data: &'a [u8],
    error_occurred: bool,
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = RiffResult<Chunk<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.error_occurred || self.cursor >= self.cursor_end {
            None
        } else {
            match Chunk::from_raw_u8(self.data, self.cursor) {
                Ok(chunk) => {
                    self.cursor = self.cursor + 8 + chunk.payload_len + (chunk.payload_len % 2);
                    Some(Ok(chunk))
                }
                Err(err) => {
                    self.error_occurred = true;
                    Some(Err(err))
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ChunkType {
    pub data: [u8; 4],
}

impl ChunkType {
    pub fn as_str(&self) -> RiffResult<&str> {
        Ok(std::str::from_utf8(&self.data)?)
    }

    pub fn as_bytes(&self) -> &[u8; 4] {
        &self.data
    }
}

/// Represents the data that a `Chunk` contains.
/// There are 3 possible values that any `Chunk` may hold.
#[derive(Debug)]
pub enum ChunkContent<'a> {
    /// Represents a `Chunk` that contains raw data as `&[u8]`.
    RawData(ChunkId, &'a [u8]),
    /// Represents a `Chunk` where the payload contains `ChunkType` identifier and a list of `ChunkContent`s.
    Children(ChunkId, ChunkType, Vec<ChunkContent<'a>>),
    /// Represents a `Chunk` where the payload only contain a list of `ChunkContent`s.
    ChildrenNoType(ChunkId, Vec<ChunkContent<'a>>),
}

/// Since `Chunk` is an opaque type. The only way to obtain the `Chunk`'s contents is through this trait.
impl<'a> TryFrom<Chunk<'a>> for ChunkContent<'a> {
    type Error = RiffError;

    fn try_from(chunk: Chunk<'a>) -> RiffResult<Self> {
        match chunk.id().as_str() {
            Ok(RIFF_ID) | Ok(LIST_ID) => {
                let chunk_type = chunk.chunk_type()?;
                let child_contents = chunk
                    .iter()
                    .map(|child| ChunkContent::try_from(child?))
                    .collect::<RiffResult<Vec<_>>>()?;
                Ok(ChunkContent::Children(
                    chunk.id(),
                    chunk_type,
                    child_contents,
                ))
            }
            Ok(SEQT_ID) => {
                let child_contents = chunk
                    .iter()
                    .map(|child| ChunkContent::try_from(child?))
                    .collect::<RiffResult<Vec<_>>>()?;
                Ok(ChunkContent::ChildrenNoType(chunk.id(), child_contents))
            }
            _ => {
                let contents = chunk.get_raw_child()?;
                Ok(ChunkContent::RawData(chunk.id(), contents))
            }
        }
    }
}

#[derive(Debug)]
pub struct ChunkId {
    pub data: [u8; 4],
}

impl ChunkId {
    pub fn as_str(&self) -> RiffResult<&str> {
        Ok(std::str::from_utf8(&self.data)?)
    }

    pub fn as_bytes(&self) -> &[u8; 4] {
        &self.data
    }
}
