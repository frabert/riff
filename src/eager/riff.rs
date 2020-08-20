use crate::error::{ChunkTooSmall, ChunkTooSmallForChunkType, PayloadLenMismatch, RiffError};
use crate::{
    constants::{LIST_ID, RIFF_ID, SEQT_ID},
    error::RiffResult,
    FourCC,
};
use std::convert::TryFrom;

/// An eager representation of a RIFF file.
#[allow(dead_code)]
#[derive(Debug)]
pub struct RiffRam {
    data: Vec<u8>,
}

impl<'a> TryFrom<&'a RiffRam> for ChunkRam<'a> {
    type Error = RiffError;

    fn try_from(value: &'a RiffRam) -> RiffResult<Self> {
        Ok(ChunkRam::from_raw_u8(&value.data, 0)?)
    }
}

#[allow(dead_code)]
impl RiffRam {
    pub fn id(&self) -> FourCC {
        let mut buff: [u8; 4] = [0; 4];
        buff.copy_from_slice(&self.data[0..4]);
        FourCC { data: buff }
    }

    pub fn payload_len(&self) -> u32 {
        let mut buff: [u8; 4] = [0; 4];
        buff.copy_from_slice(&self.data[4..8]);
        u32::from_le_bytes(buff)
    }

    pub fn iter(&self) -> RiffResult<ChunkRamIter> {
        Ok(ChunkRam::from_raw_u8(self.data.as_slice(), 0)?.iter())
    }

    pub fn from_file<T>(path: T) -> RiffResult<Self>
    where
        T: Into<std::path::PathBuf>,
    {
        let path = path.into();
        let data = std::fs::read(path)?;
        if data.len() >= 8 {
            Ok(RiffRam { data })
        } else {
            Err(RiffError::ChunkTooSmall(ChunkTooSmall { data, pos: 0 }))
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

/// Represents a chunk inside a RIFF file that have been created eagerly.
/// To create a lazy version, please see the `ChunkRam` created by `RiffDisk`.
/// Note that this is an opaque type, to obtain its content, one must convert it into a `ChunkRamContent`.
#[derive(Debug, Clone, PartialEq)]
pub struct ChunkRam<'a> {
    pos: u32,
    payload_len: u32,
    data: &'a [u8],
}

impl<'a> ChunkRam<'a> {
    pub fn id(&self) -> FourCC {
        let pos = self.pos as usize;
        let mut buff: [u8; 4] = [0; 4];
        // SAFETY: Any creation of `ChunkRam` must occur through `ChunkRam::from_raw_u8`.
        // In there, we should already checked that the `data[pos..].len()` is _at least_ 8 bytes long.
        buff.copy_from_slice(&self.data[pos..pos + 4]);
        FourCC { data: buff }
    }

    pub fn payload_len(&self) -> u32 {
        self.payload_len
    }

    pub fn from_raw_u8(data: &[u8], pos: u32) -> RiffResult<ChunkRam> {
        let pos = pos as usize;
        if data.len() >= pos + 8 {
            let mut payload_buff: [u8; 4] = [0; 4];
            payload_buff.copy_from_slice(&data[pos + 4..pos + 8]);
            Ok(ChunkRam {
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

    pub fn chunk_type(&self) -> RiffResult<FourCC> {
        let pos = self.pos as usize;
        if self.data.len() >= pos + 12 {
            let mut buff: [u8; 4] = [0; 4];
            buff.copy_from_slice(&self.data[pos + 8..pos + 12]);
            Ok(FourCC { data: buff })
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

    pub fn iter(&self) -> ChunkRamIter<'a> {
        match self.id().as_str() {
            Ok(RIFF_ID) | Ok(LIST_ID) => ChunkRamIter {
                cursor: self.pos + 12,
                // We have to subtract because RIFF_ID and LIST_ID contain chunk type that consumes 4 bytes.
                cursor_end: self.pos + 12 + self.payload_len - 4,
                data: self.data,
                error_occurred: false,
            },
            _ => ChunkRamIter {
                cursor: self.pos + 8,
                cursor_end: self.pos + 8 + self.payload_len,
                data: self.data,
                error_occurred: false,
            },
        }
    }
}

/// An iterator over the children of a `ChunkRam`.
#[derive(Debug)]
pub struct ChunkRamIter<'a> {
    cursor: u32,
    cursor_end: u32,
    data: &'a [u8],
    error_occurred: bool,
}

impl<'a> Iterator for ChunkRamIter<'a> {
    type Item = RiffResult<ChunkRam<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.error_occurred || self.cursor >= self.cursor_end {
            None
        } else {
            match ChunkRam::from_raw_u8(self.data, self.cursor) {
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

/// Represents the data that a `ChunkRam` contains.
/// There are 3 possible values that any `ChunkRam` may hold.
#[derive(Debug)]
pub enum ChunkRamContent<'a> {
    /// Represents a `ChunkRam` that contains raw data as `&[u8]`.
    RawData(FourCC, &'a [u8]),
    /// Represents a `ChunkRam` where the payload contains `FourCC` identifier and a list of `ChunkRamContent`s.
    Children(FourCC, FourCC, Vec<ChunkRamContent<'a>>),
    /// Represents a `ChunkRam` where the payload only contain a list of `ChunkRamContent`s.
    ChildrenNoType(FourCC, Vec<ChunkRamContent<'a>>),
}

/// Since `ChunkRam` is an opaque type. The only way to obtain the `ChunkRam`'s contents is through this trait.
impl<'a> TryFrom<ChunkRam<'a>> for ChunkRamContent<'a> {
    type Error = RiffError;

    fn try_from(chunk: ChunkRam<'a>) -> RiffResult<Self> {
        match chunk.id().as_str() {
            Ok(RIFF_ID) | Ok(LIST_ID) => {
                let chunk_type = chunk.chunk_type()?;
                let child_contents = chunk
                    .iter()
                    .map(|child| ChunkRamContent::try_from(child?))
                    .collect::<RiffResult<Vec<_>>>()?;
                Ok(ChunkRamContent::Children(
                    chunk.id(),
                    chunk_type,
                    child_contents,
                ))
            }
            Ok(SEQT_ID) => {
                let child_contents = chunk
                    .iter()
                    .map(|child| ChunkRamContent::try_from(child?))
                    .collect::<RiffResult<Vec<_>>>()?;
                Ok(ChunkRamContent::ChildrenNoType(chunk.id(), child_contents))
            }
            _ => {
                let contents = chunk.get_raw_child()?;
                Ok(ChunkRamContent::RawData(chunk.id(), contents))
            }
        }
    }
}
