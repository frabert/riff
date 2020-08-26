use std::convert::TryFrom;
use std::fmt::Debug;
use std::io::{Read, Seek};
use std::path::PathBuf;

use crate::{
    constants::{LIST_ID, RIFF_ID, SEQT_ID},
    error::{RiffError, RiffResult},
    FourCC,
};

/// Represents the possible data contained in a `ChunkDisk`.
#[derive(Debug)]
pub enum ChunkDiskContent {
    RawData(FourCC, Vec<u8>),
    Children(FourCC, FourCC, Vec<ChunkDiskContent>),
    ChildrenNoType(FourCC, Vec<ChunkDiskContent>),
}

/// `ChunkDisk` is an opaque type. The only way to access its content is by converting it into
/// a `ChunkDiskContent`.
impl TryFrom<ChunkDisk> for ChunkDiskContent {
    type Error = RiffError;

    /// Performs the conversion.
    fn try_from(chunk: ChunkDisk) -> RiffResult<Self> {
        let chunk_id = chunk.id().clone();
        match chunk_id.as_str() {
            Ok(RIFF_ID) | Ok(LIST_ID) => {
                let chunk_type = chunk.chunk_type();
                let child_contents = chunk
                    .iter()?
                    .map(|child| ChunkDiskContent::try_from(child?))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ChunkDiskContent::Children(
                    chunk_id,
                    chunk_type.clone()?,
                    child_contents,
                ))
            }
            Ok(SEQT_ID) => {
                let child_contents = chunk
                    .iter()?
                    .map(|child| ChunkDiskContent::try_from(child?))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ChunkDiskContent::ChildrenNoType(chunk_id, child_contents))
            }
            _ => {
                let contents = chunk.get_raw_child()?;
                Ok(ChunkDiskContent::RawData(chunk_id.clone(), contents))
            }
        }
    }
}

/// Represents a lazy reader of a chunk in a RIFF file.
#[derive(Debug)]
pub struct ChunkDisk {
    id: FourCC,
    chunk_type: Option<FourCC>,
    pos: u32,
    payload_len: u32,
    path: PathBuf,
}

impl ChunkDisk {
    pub fn id(&self) -> &FourCC {
        &self.id
    }

    pub fn payload_len(&self) -> u32 {
        self.payload_len
    }

    pub fn chunk_type(&self) -> &Option<FourCC> {
        &self.chunk_type
    }

    fn from_path(path: PathBuf, pos: u32) -> RiffResult<ChunkDisk> {
        let pos = pos as u64;
        let mut inner_reader = std::fs::File::open(&path)?;
        let id_buff = ChunkDisk::read_4_bytes(&mut inner_reader, pos)?;
        let payload_len_buff = ChunkDisk::read_4_bytes(&mut inner_reader, pos + 4)?;
        let chunk_type = match ChunkDisk::read_4_bytes(&mut inner_reader, pos + 8) {
            Ok(result) => Some(FourCC { data: result }),
            Err(_) => None,
        };
        Ok(ChunkDisk {
            id: FourCC { data: id_buff },
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
        Ok(match self.id().as_str() {
            Ok(RIFF_ID) | Ok(LIST_ID) => 12,
            _ => 8,
        })
    }

    pub fn iter(&self) -> RiffResult<ChunkDiskIter> {
        Ok(match self.id().as_str() {
            Ok(LIST_ID) | Ok(RIFF_ID) => ChunkDiskIter {
                cursor: self.pos + 12,
                cursor_end: self.pos + 12 + self.payload_len - 4,
                path: self.path.clone(),
                error_occurred: false,
            },
            _ => ChunkDiskIter {
                cursor: self.pos + 8,
                cursor_end: self.pos + 8 + self.payload_len,
                path: self.path.clone(),
                error_occurred: false,
            },
        })
    }
}

#[derive(Debug)]
pub struct ChunkDiskIter {
    cursor: u32,
    cursor_end: u32,
    path: PathBuf,
    error_occurred: bool,
}

impl Iterator for ChunkDiskIter {
    type Item = RiffResult<ChunkDisk>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.error_occurred || self.cursor >= self.cursor_end {
            None
        } else {
            match ChunkDisk::from_path(self.path.clone(), self.cursor) {
                Ok(chunk) => {
                    self.cursor = self.cursor + 8 + chunk.payload_len + (chunk.payload_len % 2);
                    Some(Ok(chunk))
                }
                // TODO: How to propagate this kind of error?
                // We need to correctly parse chunks to proceed this iterator, but we obviously failed
                // So we would like to stop...but also give the user that something wrong happened,
                // and its not just the end of file.
                Err(err) => {
                    self.error_occurred = true;
                    Some(Err(err))
                }
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RiffDisk {
    path: PathBuf,
}

impl TryFrom<RiffDisk> for ChunkDisk {
    type Error = RiffError;

    fn try_from(value: RiffDisk) -> Result<Self, Self::Error> {
        ChunkDisk::from_path(value.path, 0)
    }
}

#[allow(dead_code)]
impl RiffDisk {
    pub fn from_file<T>(path: T) -> RiffResult<Self>
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
            RIFF_ID => Ok(RiffDisk { path }),
            _ => Err(RiffError::InvalidRiffHeader),
        }
    }
}
