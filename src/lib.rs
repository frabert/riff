//! # riff
//!
//! `riff` provides utility methods for reading and writing RIFF-formatted files,
//! such as Microsoft Wave, AVI or DLS files.
//! Please refer to the specification as provided here [http://www-mmsp.ece.mcgill.ca/Documents/AudioFormats/WAVE/Docs/riffmci.pdf].
//! Also check out the Wikipedia page for the RIFF file format [https://en.wikipedia.org/wiki/Resource_Interchange_File_Format]
//! TODO: I think we should always check for compliance and provides an alternative `unchecked_*` version that removes all the compliance checks.

use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

/// A chunk id, also known as FourCC
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ChunkId {
    /// The raw bytes of the id
    pub value: [u8; 4],
}

/// The `RIFF` id
pub static RIFF_ID: ChunkId = ChunkId {
    value: [0x52, 0x49, 0x46, 0x46],
};

/// The `LIST` id
pub static LIST_ID: ChunkId = ChunkId {
    value: [0x4C, 0x49, 0x53, 0x54],
};

/// The `seqt` id
pub static SEQT_ID: ChunkId = ChunkId {
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
pub enum ChunkContents {
    RawData(ChunkId, Vec<u8>),
    Children(ChunkId, ChunkId, Vec<ChunkContents>),
    ChildrenNoType(ChunkId, Vec<ChunkContents>),
}

impl ChunkContents {
    pub fn write<T>(&self, writer: &mut T) -> std::io::Result<u64>
    where
        T: Seek + Write,
    {
        match &self {
            &ChunkContents::RawData(id, data) => {
                if data.len() as u64 > u32::MAX as u64 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Data too big",
                    ));
                }

                let len = data.len() as u32;
                writer.write_all(&id.value)?;
                writer.write_all(&len.to_le_bytes())?;
                writer.write_all(&data)?;
                if len % 2 != 0 {
                    let single_byte: [u8; 1] = [0];
                    writer.write_all(&single_byte)?;
                }
                Ok((8 + len + (len % 2)).into())
            }
            &ChunkContents::Children(id, chunk_type, children) => {
                writer.write_all(&id.value)?;
                let len_pos = writer.seek(SeekFrom::Current(0))?;
                let zeros: [u8; 4] = [0, 0, 0, 0];
                writer.write_all(&zeros)?;
                writer.write_all(&chunk_type.value)?;
                let mut total_len: u64 = 4;
                for child in children {
                    total_len = total_len + child.write(writer)?;
                }

                if total_len > u32::MAX as u64 {
                    use std::io::{Error, ErrorKind};
                    return Err(Error::new(ErrorKind::InvalidData, "Data too big"));
                }

                let end_pos = writer.seek(SeekFrom::Current(0))?;
                writer.seek(SeekFrom::Start(len_pos))?;
                writer.write_all(&(total_len as u32).to_le_bytes())?;
                writer.seek(SeekFrom::Start(end_pos))?;

                Ok((8 + total_len + (total_len % 2)).into())
            }
            &ChunkContents::ChildrenNoType(id, children) => {
                writer.write_all(&id.value)?;
                let len_pos = writer.seek(SeekFrom::Current(0))?;
                let zeros: [u8; 4] = [0, 0, 0, 0];
                writer.write_all(&zeros)?;
                let mut total_len: u64 = 0;
                for child in children {
                    total_len = total_len + child.write(writer)?;
                }

                if total_len > u32::MAX as u64 {
                    use std::io::{Error, ErrorKind};
                    return Err(Error::new(ErrorKind::InvalidData, "Data too big"));
                }

                let end_pos = writer.seek(SeekFrom::Current(0))?;
                writer.seek(SeekFrom::Start(len_pos))?;
                writer.write_all(&(total_len as u32).to_le_bytes())?;
                writer.seek(SeekFrom::Start(end_pos))?;

                Ok((8 + total_len + (total_len % 2)).into())
            }
        }
    }
}

/// A chunk, also known as a form
#[derive(PartialEq, Eq, Debug)]
pub struct Chunk {
    pos: u64,
    id: ChunkId,
    len: u32,
}

/// An iterator over the children of a `Chunk`
pub struct ChunkIter<'a, T>
where
    T: Seek + Read,
{
    end: u64,
    cur: u64,
    stream: &'a mut T,
}

impl<'a, T> Iterator for ChunkIter<'a, T>
where
    T: Seek + Read,
{
    type Item = Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur >= self.end {
            None
        } else {
            if let Ok(chunk) = Chunk::read(&mut self.stream, self.cur) {
                let len = chunk.len() as u64;
                self.cur = self.cur + len + 8 + (len % 2);
                Some(chunk)
            } else {
                None
            }
        }
    }
}

impl Chunk {
    /// Returns the `ChunkId` of this chunk.
    pub fn id(&self) -> &ChunkId {
        &self.id
    }

    /// Returns the number of bytes in this chunk.
    pub fn len(&self) -> u32 {
        self.len
    }

    /// Returns the offset of this chunk from the start of the stream.
    pub fn offset(&self) -> u64 {
        self.pos
    }

    /// Reads the chunk type of this chunk.
    ///
    /// This is not to be confused with the chunk's identifier.
    /// The type of a chunk is contained in its data field.
    /// This function makes no guarantee that the returned `ChunkId` is valid and/or appropriate.
    /// This method is generally only valid for chunk with `RIFF` and `LIST` identifiers.
    pub fn read_type<R>(&self, stream: &mut R) -> std::io::Result<ChunkId>
    where
        R: Read + Seek,
    {
        stream.seek(SeekFrom::Start(self.pos + 8))?;

        let mut fourcc: [u8; 4] = [0; 4];
        stream.read_exact(&mut fourcc)?;

        Ok(ChunkId { value: fourcc })
    }

    /// Reads a chunk from the specified position in the stream.
    pub fn read<T>(stream: &mut T, pos: u64) -> std::io::Result<Chunk>
    where
        T: Read + Seek,
    {
        stream.seek(SeekFrom::Start(pos))?;

        let mut fourcc: [u8; 4] = [0; 4];
        stream.read_exact(&mut fourcc)?;

        let mut len: [u8; 4] = [0; 4];
        stream.read_exact(&mut len)?;

        Ok(Chunk {
            pos,
            id: ChunkId { value: fourcc },
            len: u32::from_le_bytes(len),
        })
    }

    /// Reads the entirety of the contents of a chunk.
    pub fn read_contents<T>(&self, stream: &mut T) -> std::io::Result<Vec<u8>>
    where
        T: Read + Seek,
    {
        stream.seek(SeekFrom::Start(self.pos + 8))?;

        let mut data: Vec<u8> = vec![0; self.len as usize];
        stream.read_exact(&mut data)?;

        Ok(data)
    }

    /// Returns an iterator over the children of the chunk.
    /// This method assumes that the file is `RIFF` compliant.
    /// That is, it must provide an identifier for the child chunk.
    /// It is the responsibility of the caller to ensure that the data
    /// contained are subchunks.
    ///
    /// If the parent chunk's identifier is either "RIFF" or "LIST",
    /// then it is a chunk that contain subchunks.
    /// The `RIFF` and `LIST` chunk data have the following format:
    ///     1. 4 bytes of ASCII identifier for this particular `RIFF` or
    ///        `LIST` chunk.
    ///     2, Rest of the data.
    /// Note that this could apply recursively.
    pub fn iter<'a, T>(&self, stream: &'a mut T) -> ChunkIter<'a, T>
    where
        T: Seek + Read,
    {
        ChunkIter {
            // The offset of 12 comes from.
            // 4 bytes ASCII identifier of parent chunk
            // 4 bytes LE integer of this schunk
            // 4 bytes ASCII identifer of child chunk
            cur: self.pos + 12,
            end: self.pos + 4 + (self.len as u64),
            stream: stream,
        }
    }

    /// Returns an iterator over the chilren of the chunk. Only valid for
    /// noncompliant chunks that have children but no chunk type identifier
    /// (like `seqt` chunks).
    pub fn iter_no_type<'a, T>(&self, stream: &'a mut T) -> ChunkIter<'a, T>
    where
        T: Seek + Read,
    {
        ChunkIter {
            cur: self.pos + 8,
            end: self.pos + 4 + (self.len as u64),
            stream: stream,
        }
    }
}

pub  struct Riff {
    
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
