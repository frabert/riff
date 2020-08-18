use crate::{
    constants::{LIST_ID, RIFF_ID, SEQT_ID},
    error::RiffResult,
};

/// An eager representation of a RIFF file.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Riff {
    data: Vec<u8>,
}

impl<'a> From<&'a Riff> for Chunk<'a> {
    fn from(value: &'a Riff) -> Self {
        Chunk::from_raw_u8(&value.data, 0)
    }
}

#[allow(dead_code)]
impl Riff {
    pub fn id(&self) -> ChunkId {
        let mut buff: [u8; 4] = [0; 4];
        buff.copy_from_slice(&self.data[0..4]);
        ChunkId { value: buff }
    }

    pub fn payload_len(&self) -> u32 {
        let mut buff: [u8; 4] = [0; 4];
        buff.copy_from_slice(&self.data[4..8]);
        u32::from_le_bytes(buff)
    }

    pub fn iter(&self) -> ChunkIter {
        Chunk::from_raw_u8(self.data.as_slice(), 0).iter()
    }

    pub fn from_file(path: std::path::PathBuf) -> std::io::Result<Self> {
        let data = std::fs::read(path)?;
        Ok(Riff { data })
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
        buff.copy_from_slice(&self.data[pos..pos + 4]);
        ChunkId { value: buff }
    }

    pub fn payload_len(&self) -> u32 {
        self.payload_len
    }

    pub fn from_raw_u8(data: &[u8], pos: u32) -> Chunk {
        let pos = pos as usize;
        let mut payload_buff: [u8; 4] = [0; 4];
        payload_buff.copy_from_slice(&data[pos + 4..pos + 8]);
        Chunk {
            pos: pos as u32,
            payload_len: u32::from_le_bytes(payload_buff),
            data,
        }
    }

    pub fn chunk_type(&self) -> ChunkType {
        let pos = self.pos as usize;
        let mut buff: [u8; 4] = [0; 4];
        buff.copy_from_slice(&self.data[pos + 8..pos + 12]);
        ChunkType { value: buff }
    }

    pub fn get_raw_child(&self) -> &'a [u8] {
        let pos = self.pos as usize;
        let payload_len = self.payload_len as usize;
        let offset = match self.id().as_str() {
            Ok(RIFF_ID) | Ok(LIST_ID) => 12,
            _ => 8,
        };
        &self.data[pos + offset..pos + offset + payload_len]
    }

    pub fn iter(&self) -> ChunkIter<'a> {
        match self.id().as_str() {
            Ok(RIFF_ID) | Ok(LIST_ID) => ChunkIter {
                cursor: self.pos + 12,
                // We have to subtract because RIFF_ID and LIST_ID contain chunk type that consumes 4 bytes.
                cursor_end: self.pos + 12 + self.payload_len - 4,
                data: self.data,
            },
            _ => ChunkIter {
                cursor: self.pos + 8,
                cursor_end: self.pos + 8 + self.payload_len,
                data: self.data,
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
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = Chunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.cursor_end {
            None
        } else {
            let chunk = Chunk::from_raw_u8(self.data, self.cursor);
            self.cursor = self.cursor + 8 + chunk.payload_len + (chunk.payload_len % 2);
            Some(chunk)
        }
    }
}

#[derive(Debug)]
pub struct ChunkType {
    pub value: [u8; 4],
}

impl ChunkType {
    pub fn as_str(&self) -> RiffResult<&str> {
        Ok(std::str::from_utf8(&self.value)?)
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
impl<'a> From<Chunk<'a>> for ChunkContent<'a> {
    fn from(chunk: Chunk<'a>) -> Self {
        match chunk.id().as_str() {
            Ok(RIFF_ID) | Ok(LIST_ID) => {
                let chunk_type = chunk.chunk_type();
                let child_contents = chunk
                    .iter()
                    .map(|child| ChunkContent::from(child))
                    .collect();
                ChunkContent::Children(chunk.id(), chunk_type, child_contents)
            }
            Ok(SEQT_ID) => {
                let child_contents = chunk
                    .iter()
                    .map(|child| ChunkContent::from(child))
                    .collect();
                ChunkContent::ChildrenNoType(chunk.id(), child_contents)
            }
            _ => {
                let contents = chunk.get_raw_child();
                ChunkContent::RawData(chunk.id(), contents)
            }
        }
    }
}

#[derive(Debug)]
pub struct ChunkId {
    pub value: [u8; 4],
}

impl ChunkId {
    pub fn as_str(&self) -> RiffResult<&str> {
        // TODO: Propagate this error.
        Ok(std::str::from_utf8(&self.value)?)
    }
}
