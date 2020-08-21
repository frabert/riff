use crate::constants::RIFF_ID;

use crate::{
    error::{RiffError, RiffResult},
    FourCC,
};

/// Helper struct to help user creates chunks.
///
/// # NOTE
///
/// 1. Add lots and lots of error checking. Since `Vec` use `usize` internally, we have to limit it to `u32`.
///    In similar vein, we also need to check for overflow and stuff, especially during subtraction.
/// 2. Should we separate Type and NoType? We currently represent these types at the most raw level (literally as `Vec<u8>`) which is quite hard to deal with.
/// 3. Should we lazily calculate the size of the payload or do we want to precalculate the payload as we append chunks.
///    The latter method is error prone because a user could append to an inner chunk and the parent chunk would have no way of knowing about this size increase.
#[derive(Debug)]
pub struct ChunkBuilder {
    pub chunk_id: FourCC,
    pub payload_len: u32,
    pub chunk_type: Option<FourCC>,
    pub data: ChunkData,
}

impl ChunkBuilder {
    /// Creates a chunk from a `FourCC` and a `ChunkData` that does not contain a 4 bytes identifier for the chunk type.
    pub fn new_notype(id: FourCC, data: ChunkData) -> Self {
        ChunkBuilder {
            chunk_id: id,
            payload_len: ChunkBuilder::calculate_payload(&data, 0),
            chunk_type: None,
            data: ChunkBuilder::fit_data(data),
        }
    }

    /// Creates a chunk from 2 `FourCC`s and a `ChunkData` that uses the second `FourCC` and the chunk's type identifier.
    pub fn new_type(id: FourCC, chunk_type: FourCC, data: ChunkData) -> Self {
        ChunkBuilder {
            chunk_id: id,
            payload_len: ChunkBuilder::calculate_payload(&data, 4),
            chunk_type: Some(chunk_type),
            data: ChunkBuilder::fit_data(data),
        }
    }

    /// Adds a chunk children to this chunk.
    pub fn add_chunk(mut self, chunk: ChunkBuilder) -> RiffResult<Self> {
        self.payload_len += 8;
        if chunk.chunk_type.is_some() {
            self.payload_len += 4;
        }
        match self.data {
            ChunkData::RawData(_) => return Err(RiffError::MismatchChunkAdded),
            ChunkData::ChunkList(ref mut vec) => {
                self.payload_len += vec.iter().map(|x| x.payload_len + 8).sum::<u32>();
                vec.push(chunk);
            }
        }
        Ok(self)
    }

    fn calculate_payload(data: &ChunkData, offset: u32) -> u32 {
        let payload_len = match &data {
            ChunkData::ChunkList(data) => data
                .iter()
                .map(|x| {
                    if x.chunk_type.is_none() {
                        x.payload_len + 8
                    } else {
                        x.payload_len + 4 + 8
                    }
                })
                .sum(),
            ChunkData::RawData(data) => data.len() as u32,
        };
        payload_len + offset
    }

    fn fit_data(data: ChunkData) -> ChunkData {
        match data {
            ChunkData::RawData(mut vec) => {
                if vec.len() % 2 == 1 {
                    vec.push(0);
                }
                ChunkData::RawData(vec)
            }
            chunks => chunks,
        }
    }

    fn to_bytes<'a>(&self, mut result: &'a mut Vec<u8>) -> &'a Vec<u8> {
        result.extend_from_slice(self.chunk_id.as_bytes());
        result.extend_from_slice(&self.payload_len.to_le_bytes());
        if self.chunk_type.is_some() {
            result.extend_from_slice(self.chunk_type.as_ref().unwrap().as_bytes());
        }
        match &self.data {
            ChunkData::RawData(raw) => result.extend_from_slice(&raw),
            ChunkData::ChunkList(chunks) => {
                for x in chunks {
                    x.to_bytes(&mut result);
                }
            }
        }
        result
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct RiffBuilder {
    pub payload_len: u32,
    pub chunk_type: FourCC,
    pub data: Vec<ChunkBuilder>,
}

impl RiffBuilder {
    pub fn new(chunk_type: FourCC) -> Self {
        RiffBuilder {
            payload_len: 4,
            chunk_type,
            data: Vec::new(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(RIFF_ID.as_bytes());
        result.extend_from_slice(&self.payload_len.to_le_bytes());
        result.extend_from_slice(&self.chunk_type.data);
        for x in &self.data {
            x.to_bytes(&mut result);
        }
        result
    }

    pub fn add_chunk(mut self, chunk: ChunkBuilder) -> Self {
        self.payload_len += 8;
        if chunk.chunk_type.is_some() {
            self.payload_len += 4;
        }
        match chunk.data {
            ChunkData::RawData(ref raw) => {
                self.payload_len += raw.len() as u32;
            }
            ChunkData::ChunkList(ref vec) => {
                self.payload_len += vec.iter().map(|x| x.payload_len + 8).sum::<u32>()
            }
        }
        self.data.push(chunk);
        self
    }
}

#[derive(Debug)]
pub enum ChunkData {
    RawData(Vec<u8>),
    ChunkList(Vec<ChunkBuilder>),
}
