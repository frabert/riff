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
    /// The raw bytes of the id
    pub value: [u8; 4],
}

/// The `RIFF` id
pub const RIFF_ID: ChunkId = ChunkId {
    value: [0x52, 0x49, 0x46, 0x46],
};

/// The `LIST` id
pub const LIST_ID: ChunkId = ChunkId {
    value: [0x4C, 0x49, 0x53, 0x54],
};

/// The `seqt` id
pub const SEQT_ID: ChunkId = ChunkId {
    value: [0x73, 0x65, 0x71, 0x74],
};

impl ChunkId {
    /// Returns the value of the id as a string.
    ///
    /// # Examples
    /// ```
    /// assert_eq!(riff::RIFF_ID.as_str(), "RIFF");
    /// ```
    ///
    /// # Panics
    /// This function panics when the value does not represent a valid UTF-8 string.
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.value).unwrap()
    }

    /// Creates a new ChunkId from a string.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), (Box<dyn Error>)> {
    /// let chunk_id = riff::ChunkId::new("RIFF")?;
    /// #   Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    ///
    /// # Errors
    /// The function fails when the string's length in bytes is not exactly 4.
    pub fn new(s: &str) -> Result<ChunkId, &str> {
        let bytes = s.as_bytes();
        if bytes.len() != 4 {
            Err("Invalid length")
        } else {
            let mut a: [u8; 4] = Default::default();
            a.copy_from_slice(&bytes[..]);
            Ok(ChunkId { value: a })
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum ChunkContents<'a> {
    RawData(ChunkId, &'a [u8]),
    Children(ChunkId, ChunkId, Vec<ChunkContents<'a>>),
    ChildrenNoType(ChunkId, Vec<ChunkContents<'a>>),
}

impl<'a> From<Chunk> for ChunkContents<'a> {
    fn from(chunk: Chunk) -> Self {
        match chunk.id() {
            RIFF_ID | LIST_ID => {
                let child_id = chunk.get_child_id();
                let child_contents: Vec<ChunkContents<'a>> = chunk
                    .child_iter()
                    .map(|child| ChunkContents::from(child))
                    .collect();

                ChunkContents::Children(chunk.id(), child_id, child_contents)
            }
            SEQT_ID => {
                let child_contents = chunk
                    .child_iter_notype()
                    .map(|child| ChunkContents::from(child))
                    .collect();

                ChunkContents::ChildrenNoType(chunk.id(), child_contents)
            }
            _ => ChunkContents::RawData(chunk.id(), unsafe {
                std::slice::from_raw_parts(chunk.data, chunk.len as usize)
            }),
        }
    }
}

/// A chunk, also known as a form.
/// Note that `Chunk` is an opaque type.
/// To obtain the actual chunk.data, chunk.convert this into a `ChunkContents`.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pos: u32,
    len: u32,
    data: *const u8,
}

impl Chunk {
    /// Returns the `ChunkId` of this chunk.
    pub fn id(&self) -> ChunkId {
        let pos = self.pos as usize;
        let mut buff: [u8; 4] = [0; 4];
        // buff.copy_from_slice(&self.data[pos..pos + 4]);
        unsafe {
            std::ptr::copy(self.data.add(pos), buff.as_mut_ptr(), 4);
        }
        ChunkId { value: buff }
    }

    /// Returns the number of bytes in this chunk.
    pub fn len(&self) -> u32 {
        self.len
    }

    /// Returns the offset of this chunk from the start of the stream.
    pub fn offset(&self) -> u32 {
        self.pos
    }

    pub fn from_raw_u8(data: &[u8], pos: u32) -> Chunk {
        let pos = pos as usize;
        let mut buff: [u8; 4] = [0; 4];
        buff.copy_from_slice(&data[pos + 4..pos + 8]);
        Chunk {
            pos: pos as u32,
            len: u32::from_le_bytes(buff),
            data: data.as_ptr(),
        }
    }

    /// Reads the chunk type of this chunk.
    ///
    /// This is not to be confused with the chunk's identifier.
    /// The type of a chunk is contained in its data field.
    /// This function makes no guarantee that the returned `ChunkId` is valid and/or appropriate.
    /// This method is generally only valid for chunk with `RIFF` and `LIST` identifiers.
    pub fn get_child_id(&self) -> ChunkId {
        let pos = self.pos as usize;
        let mut buff: [u8; 4] = [0; 4];
        unsafe {
            std::ptr::copy(self.data.add(pos + 8), buff.as_mut_ptr(), 4);
        }
        ChunkId { value: buff }
    }

    /// Reads a chunk from the specified position in the stream.
    pub fn get_child_chunk(&self) -> Chunk {
        let pos = self.pos as usize;
        let mut buff: [u8; 4] = [0; 4];
        // buff.copy_from_slice(&self.data[pos + 4..pos + 8]);
        unsafe {
            std::ptr::copy(self.data.add(pos + 4), buff.as_mut_ptr(), 4);
        }
        Chunk {
            pos: self.pos + self.len,
            len: u32::from_le_bytes(buff),
            data: self.data,
        }
    }

    /// Reads the entirety of the contents of a chunk as `u8` excluding
    /// the child's ASCII identifier.
    pub fn get_child_content(&self) -> &[u8] {
        let pos = self.pos as usize;
        let len = self.len as usize;
        //  &self.data[pos + 12..pos + len]
        unsafe { std::slice::from_raw_parts(self.data.add(pos + 12), len) }
    }

    /// Reads the entirety of the contents of a chunk as `u8`.
    pub fn get_child_content_untyped<T>(&self) -> &[u8] {
        let pos = self.pos as usize;
        let len = self.len as usize;
        // &self.data[pos + 8..pos + len]
        unsafe { std::slice::from_raw_parts(self.data.add(pos + 8), len) }
    }

    /// Creates an iterator of the childrens.
    pub fn child_iter(&self) -> ChunkIter {
        ChunkIter {
            cursor: self.pos + 12,
            end: self.len,
            data: self.data,
        }
    }

    /// Creates an iterator of the childrens.
    pub fn child_iter_notype(&self) -> ChunkIter {
        ChunkIter {
            cursor: self.pos + 8,
            end: self.len,
            data: self.data,
        }
    }
}

#[derive(Debug)]
pub struct ChunkIter {
    cursor: u32,
    end: u32,
    data: *const u8,
}

impl Iterator for ChunkIter {
    type Item = Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.end {
            None
        } else {
            let chunk = Chunk::from_raw_u8(
                unsafe { std::slice::from_raw_parts(self.data, self.end as usize) },
                self.cursor,
            );
            self.cursor = self.cursor + 8 + chunk.len + (chunk.len % 2);
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
    pub fn get_chunk(&self) -> Chunk {
        Chunk {
            pos: 0,
            len: self.data.len() as u32,
            data: self.data.as_ptr(),
        }
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

        assert_eq!(ChunkId::new("123"), Err("Invalid length"));
        assert_eq!(ChunkId::new("12345"), Err("Invalid length"));
    }

    #[test]
    fn chunkid_to_str() {
        assert_eq!(RIFF_ID.as_str(), "RIFF");
        assert_eq!(LIST_ID.as_str(), "LIST");
        assert_eq!(SEQT_ID.as_str(), "seqt");
        assert_eq!(ChunkId::new("123 ").unwrap().as_str(), "123 ");
    }
}
