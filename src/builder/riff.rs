use crate::constants::RIFF_ID;
use crate::error::RiffResult;
use crate::{ChunkId, ChunkType};

pub struct Chunk {
    data: Vec<u8>,
}

impl Chunk {
    pub fn new(chunk_id: ChunkId, payload_data: Vec<u8>) -> Self {
        let data = chunk_id
            .as_bytes()
            .iter()
            .chain(
                // TODO: Error check if payload is larger than U32_MAX
                (payload_data.len() as u32).to_le_bytes().iter(),
            )
            .chain(payload_data.iter())
            .copied()
            .collect::<Vec<u8>>();
        Chunk { data }
    }

    pub fn with_capacity(chunk_id: ChunkType, capacity: usize) -> Self {
        let mut data = chunk_id
            .as_bytes()
            .iter()
            .chain(
                // TODO: Error check if payload is larger than U32_MAX
                (0 as u32).to_le_bytes().iter(),
            )
            .copied()
            .collect::<Vec<u8>>();
        data.reserve(capacity);
        Chunk { data }
    }

    pub fn add_chunk(chunk: Chunk) {
        unimplemented!()
    }
}

pub struct Riff {
    data: Vec<u8>,
}

impl Riff {
    pub fn new(chunk_type: ChunkType) -> Self {
        let data = RIFF_ID
            .as_bytes()
            .iter()
            .chain((chunk_type.as_bytes().len() as u32).to_le_bytes().iter())
            .chain(chunk_type.as_bytes().iter())
            .copied()
            .collect::<Vec<u8>>();
        Riff { data }
    }

    pub fn with_capacity(chunk_type: ChunkType, capacity: usize) -> Self {
        let mut data = RIFF_ID
            .as_bytes()
            .iter()
            .chain((chunk_type.as_bytes().len() as u32).to_le_bytes().iter())
            .chain(chunk_type.as_bytes().iter())
            .copied()
            .collect::<Vec<u8>>();
        data.reserve(capacity);
        Riff { data }
    }

    pub fn write(&self) -> RiffResult<()> {
        unimplemented!()
    }

    pub fn add_chunk(chunk: Chunk) {
        unimplemented!()
    }
}
