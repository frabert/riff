use crate::constants::RIFF_ID;

use crate::FourCC;

/// TODO:
/// 1. Add lots and lots of error checking. Since `Vec` use `usize` internally, we have to limit it to `u32`.
///    In similar vein, we also need to check for overflow and stuff, especially during subtraction.
/// 2. Should we separate Type and NoType? We currently represent these types at the most raw level (literally as `Vec<u8>`) which is quite hard to deal with.
pub struct Chunk {
    chunk_id: FourCC,
    payload_len: u32,
    chunk_type: Option<FourCC>,
    data: ChunkChildren,
}

impl Chunk {
    pub fn new_notype(id: FourCC, data: ChunkChildren) -> Self {
        Chunk {
            chunk_id: id,
            payload_len: Chunk::calculate_payload(&data),
            chunk_type: None,
            data: Chunk::fit_data(data),
        }
    }

    pub fn new_type(id: FourCC, chunk_type: FourCC, data: ChunkChildren) -> Self {
        Chunk {
            chunk_id: id,
            payload_len: Chunk::calculate_payload(&data),
            chunk_type: Some(chunk_type),
            data: Chunk::fit_data(data),
        }
    }

    fn calculate_payload(data: &ChunkChildren) -> u32 {
        let payload_len = match &data {
            ChunkChildren::Chunk(data) => data
                .iter()
                .map(|x| {
                    if x.chunk_type.is_none() {
                        x.payload_len + 8
                    } else {
                        x.payload_len + 4 + 8
                    }
                })
                .sum(),
            ChunkChildren::RawData(data) => data.len() as u32 + 8,
        };
        if payload_len % 2 == 1 {
            payload_len + 1
        } else {
            payload_len
        }
    }

    fn fit_data(data: ChunkChildren) -> ChunkChildren {
        match data {
            ChunkChildren::RawData(mut vec) => {
                if vec.len() % 2 == 1 {
                    vec.push(0);
                }
                ChunkChildren::RawData(vec)
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
            ChunkChildren::RawData(raw) => result.extend_from_slice(&raw),
            ChunkChildren::Chunk(chunks) => {
                for x in chunks {
                    x.to_bytes(&mut result);
                }
            }
        }
        result
    }
}

#[allow(dead_code)]
pub struct Riff {
    payload_len: u32,
    chunk_type: FourCC,
    data: Vec<Chunk>,
}

impl Riff {
    pub fn new(chunk_type: FourCC) -> Self {
        Riff {
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

    pub fn add_chunk(mut self, chunk: Chunk) -> Self {
        self.payload_len += chunk.payload_len;
        self.data.push(chunk);
        self
    }
}

pub enum ChunkChildren {
    RawData(Vec<u8>),
    Chunk(Vec<Chunk>),
}
