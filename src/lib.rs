//! # riff
//!
//! `riff` provides utility methods for reading and writing RIFF-formatted files,
//! such as Microsoft Wave, AVI or DLS files.
//! Please refer to the specification as provided here [http://www-mmsp.ece.mcgill.ca/Documents/AudioFormats/WAVE/Docs/riffmci.pdf].
//! Also check out the Wikipedia page for the RIFF file format [https://en.wikipedia.org/wiki/Resource_Interchange_File_Format]
//! TODO: I think we should always check for compliance and provides an alternative `unchecked_*` version that removes all the compliance checks.

/// A chunk id, also known as FourCC
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ChunkId {
    pub value: [u8; 4],
}

pub const RIFF_ID: ChunkId = ChunkId {
    value: [0x52, 0x49, 0x46, 0x46],
};

pub const LIST_ID: ChunkId = ChunkId {
    value: [0x4C, 0x49, 0x53, 0x54],
};

pub const SEQT_ID: ChunkId = ChunkId {
    value: [0x73, 0x65, 0x71, 0x74],
};

impl ChunkId {
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.value).unwrap()
    }

    pub fn new(s: &str) -> Option<ChunkId> {
        let bytes = s.as_bytes();
        if bytes.len() != 4 {
            None
        } else {
            let mut a: [u8; 4] = Default::default();
            a.copy_from_slice(&bytes[..]);
            Some(ChunkId { value: a })
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum ChunkContents<'a> {
    RawData(ChunkId, &'a [u8]),
    Children(ChunkId, ChunkId, Vec<ChunkContents<'a>>),
    ChildrenNoType(ChunkId, Vec<ChunkContents<'a>>),
}

impl<'a> From<Chunk<'a>> for ChunkContents<'a> {
    fn from(chunk: Chunk<'a>) -> Self {
        match chunk.id() {
            RIFF_ID | LIST_ID => {
                let child_id = chunk.get_child_id();
                let child_contents: Vec<ChunkContents<'a>> = chunk
                    .iter_type()
                    .map(|child| ChunkContents::from(child))
                    .collect();

                ChunkContents::Children(chunk.id(), child_id, child_contents)
            }
            SEQT_ID => {
                let child_contents = chunk
                    .iter_notype()
                    .map(|child| ChunkContents::from(child))
                    .collect();

                ChunkContents::ChildrenNoType(chunk.id(), child_contents)
            }
            _ => {
                let contents = chunk.get_raw_child_content_untyped();
                ChunkContents::RawData(chunk.id(), contents)
            }
        }
    }
}

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

    pub fn len(&self) -> u32 {
        self.payload_len
    }

    pub fn is_empty(&self) -> bool {
        self.payload_len == 0
    }

    pub fn offset(&self) -> u32 {
        self.pos
    }

    pub fn from_raw_u8(data: &[u8], pos: u32) -> Chunk {
        let pos = pos as usize;
        let mut buff: [u8; 4] = [0; 4];
        buff.copy_from_slice(&data[pos + 4..pos + 8]);
        Chunk {
            pos: pos as u32,
            payload_len: u32::from_le_bytes(buff),
            data: data,
        }
    }

    pub fn get_child_id(&self) -> ChunkId {
        let pos = self.pos as usize;
        let mut buff: [u8; 4] = [0; 4];
        buff.copy_from_slice(&self.data[pos + 8..pos + 12]);
        ChunkId { value: buff }
    }

    pub fn get_child_chunk_typed(&self) -> Chunk<'a> {
        Chunk::from_raw_u8(self.data, self.pos + 12)
    }

    pub fn get_child_chunk_untyped(&self) -> Chunk<'a> {
        Chunk::from_raw_u8(self.data, self.pos + 8)
    }

    pub fn get_raw_child_content_typed(&self) -> &'a [u8] {
        let pos = self.pos as usize;
        let payload_len = self.payload_len as usize;
        &self.data[pos + 12..pos + 12 + payload_len]
    }

    pub fn get_raw_child_content_untyped(&self) -> &'a [u8] {
        let pos = self.pos as usize;
        let payload_len = self.payload_len as usize;
        &self.data[pos + 8..pos + 8 + payload_len]
    }

    pub fn iter_type(&self) -> ChunkIterType<'a> {
        ChunkIterType {
            cursor: self.pos + 12,
            end: self.data.len() as u32,
            data: self.data,
        }
    }

    pub fn iter_notype(&self) -> ChunkIterNoType<'a> {
        ChunkIterNoType {
            cursor: self.pos + 8,
            end: self.data.len() as u32,
            data: self.data,
        }
    }
}

#[derive(Debug)]
pub struct ChunkIterType<'a> {
    cursor: u32,
    end: u32,
    data: &'a [u8],
}

impl<'a> Iterator for ChunkIterType<'a> {
    type Item = Chunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.end {
            None
        } else {
            let chunk = Chunk::from_raw_u8(self.data, self.cursor);
            println!(
                "next_chunk:{:?} self.cursor:{:?} chunk.payload_len:{:?} (chunk.payload_len % 2):{:?}",
                chunk,
                self.cursor,
                chunk.payload_len,
                (chunk.payload_len % 2)
            );
            self.cursor = self.cursor + 4 + 4 + 4 + chunk.payload_len + (chunk.payload_len % 2);
            println!("iterator:{:?}", self);
            Some(chunk)
        }
    }
}

#[derive(Debug)]
pub struct ChunkIterNoType<'a> {
    cursor: u32,
    end: u32,
    data: &'a [u8],
}

impl<'a> Iterator for ChunkIterNoType<'a> {
    type Item = Chunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.end {
            None
        } else {
            let chunk = Chunk::from_raw_u8(self.data, self.cursor);
            println!(
                "next_chunk:{:?} self.cursor:{:?} chunk.payload_len:{:?} (chunk.payload_len % 2):{:?}",
                chunk,
                self.cursor,
                chunk.payload_len,
                (chunk.payload_len % 2)
            );
            self.cursor = self.cursor + 4 + 4 + chunk.payload_len + (chunk.payload_len % 2);
            println!("iterator:{:?}", self);
            Some(chunk)
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Riff {
    // This should be the only instance of the RIFF data in memory.
    data: Vec<u8>,
}

#[allow(dead_code)]
impl Riff {
    pub fn len(&self) -> u32 {
        self.data.len() as u32
    }

    pub fn is_empty(&self) -> bool {
        self.data.len() == 0
    }

    pub fn get_chunk(&self) -> Chunk {
        self.data.iter().for_each(|x| {
            println!("{:#b} => {:?}", x, x);
        });
        Chunk::from_raw_u8(self.data.as_slice(), 0)
    }

    pub fn from_file(path: std::path::PathBuf) -> std::io::Result<Self> {
        let data = std::fs::read(path)?;
        Ok(Riff { data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunkid_from_str() {
        assert_eq!(ChunkId::new("RIFF").unwrap(), RIFF_ID);
        assert_eq!(ChunkId::new("LIST").unwrap(), LIST_ID);
        assert_eq!(ChunkId::new("seqt").unwrap(), SEQT_ID);

        assert_eq!(
            ChunkId::new("123 ").unwrap(),
            ChunkId {
                value: [0x31, 0x32, 0x33, 0x20]
            }
        );

        assert_eq!(ChunkId::new("123"), None);
        assert_eq!(ChunkId::new("12345"), None);
    }

    #[test]
    fn chunkid_to_str() {
        assert_eq!(RIFF_ID.as_str(), "RIFF");
        assert_eq!(LIST_ID.as_str(), "LIST");
        assert_eq!(SEQT_ID.as_str(), "seqt");
        assert_eq!(ChunkId::new("123 ").unwrap().as_str(), "123 ");
    }
}
