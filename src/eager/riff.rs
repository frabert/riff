use crate::error::{ChunkTooSmall, ChunkTooSmallForChunkType, PayloadLenMismatch, RiffError};
use crate::{
    constants::{LIST_ID, RIFF_ID, SEQT_ID},
    error::RiffResult,
    FourCC,
};
use std::convert::TryFrom;

/// An eager representation of a RIFF file.
/// This struct will read the entire file and load it into resident memory.
///
/// # Example
///
/// ```rust
/// # use riffu::eager::riff::RiffRam;
/// # use riffu::error::RiffResult;
/// # pub fn main() -> RiffResult<()>{
/// let riff = RiffRam::from_file("test_assets/set_1.riff")?;
/// assert_eq!(riff.as_bytes(), vec![82, 73, 70, 70, 14, 0, 0, 0, 115, 109, 112, 108, 116, 101, 115, 116, 1, 0, 0, 0, 255, 0]);
/// # Ok(())
/// # }
/// ```
#[allow(dead_code)]
#[derive(Debug)]
pub struct RiffRam {
    pub(crate) data: Vec<u8>,
}

/// `RiffRam` can be converted to a `ChunkRam`.
impl<'a> TryFrom<&'a RiffRam> for ChunkRam<'a> {
    type Error = RiffError;

    /// Performs the conversion.
    fn try_from(value: &'a RiffRam) -> RiffResult<Self> {
        Ok(ChunkRam::from_raw_u8(&value.data)?)
    }
}

#[allow(dead_code)]
impl RiffRam {
    /// Returns the ASCII idientifier of the RIFF file.
    /// It _should_ only return the character `RIFF` and nothing else.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use riffu::{constants::RIFF_ID, error::RiffResult, eager::riff::RiffRam};
    /// # pub fn main() -> RiffResult<()> {
    /// let riff = RiffRam::from_file("test_assets/set_1.riff")?;
    /// assert_eq!(riff.id().as_str()?, RIFF_ID);
    /// # Ok(())
    /// # }
    /// ```
    pub fn id(&self) -> FourCC {
        let mut buff: [u8; 4] = [0; 4];
        buff.copy_from_slice(&self.data[0..4]);
        FourCC { data: buff }
    }

    /// Returns the payload length of this RIFF file excluding the ASCII identifier and the 32-bits
    /// representig this payload length.
    ///
    /// # Example
    ///
    /// ```
    /// # use riffu::{constants::RIFF_ID, error::RiffResult, eager::riff::RiffRam};
    /// # pub fn main() -> RiffResult<()> {
    /// let riff = RiffRam::from_file("test_assets/set_1.riff")?;
    /// assert_eq!(riff.payload_len(), 14);
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload_len(&self) -> u32 {
        let mut buff: [u8; 4] = [0; 4];
        buff.copy_from_slice(&self.data[4..8]);
        u32::from_le_bytes(buff)
    }

    /// Returns the iterator over this chunk's children.
    ///
    /// This is just a wrapper over converting this RIFF struct into a chunk and return the chunk's
    /// iterator.
    ///
    /// # Example
    ///
    /// ```
    /// # use riffu::{constants::RIFF_ID, error::RiffResult, eager::riff::RiffRam};
    /// # pub fn main() -> RiffResult<()> {
    /// let riff = RiffRam::from_file("test_assets/set_1.riff")?;
    /// assert_eq!(riff.iter()?.next()??.id().as_str()?, "test");
    /// # Ok(())
    /// # }
    /// ```
    pub fn iter(&self) -> RiffResult<ChunkRamIter> {
        Ok(ChunkRam::try_from(self)?.iter())
    }

    /// Creates this struct from a file path.
    ///
    /// # Example
    ///
    /// ```
    /// # use riffu::{constants::RIFF_ID, error::RiffResult, eager::riff::RiffRam};
    /// # pub fn main() -> RiffResult<()> {
    /// let riff = RiffRam::from_file("test_assets/set_1.riff")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_file<T>(path: T) -> RiffResult<Self>
    where
        T: Into<std::path::PathBuf>,
    {
        let path = path.into();
        let data = std::fs::read(path)?;
        if data.len() >= 8 {
            let mut id_buff: [u8; 4] = [0; 4];
            id_buff.copy_from_slice(&data[0..4]);
            let id = FourCC { data: id_buff };
            if id.as_str()? == RIFF_ID {
                Ok(RiffRam { data })
            } else {
                Err(RiffError::InvalidRiffHeader)
            }
        } else {
            Err(RiffError::ChunkTooSmall(ChunkTooSmall { data }))
        }
    }

    /// Returns the content of this RIFF file as a slice of bytes.
    ///
    /// # Example
    ///
    /// ```
    /// # use riffu::{constants::RIFF_ID, error::RiffResult, eager::riff::RiffRam};
    /// # pub fn main() -> RiffResult<()> {
    /// let riff = RiffRam::from_file("test_assets/set_1.riff")?;
    /// assert_eq!(riff.as_bytes(), &[82, 73, 70, 70, 14, 0, 0, 0, 115, 109, 112, 108, 116, 101, 115, 116, 1, 0, 0, 0, 255, 0]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

/// Represents a chunk inside a RIFF file that have been created eagerly.
/// To create a lazy version, please see the `ChunkRam` created by `RiffDisk`.
/// Note that this is an opaque type, to obtain its content, one must convert it into a `ChunkRamContent`.
#[derive(Debug, Clone, PartialEq)]
pub struct ChunkRam<'a> {
    data: &'a [u8],
}

/// Implementation of `ChunkRam`.
impl<'a> ChunkRam<'a> {
    /// Returns the ASCII identifier.
    pub fn id(&self) -> FourCC {
        let mut buff: [u8; 4] = [0; 4];
        // SAFETY: Any creation of `ChunkRam` must occur through `ChunkRam::from_raw_u8`.
        // In there, we should already checked that the `data[pos..].len()` is _at least_ 8 bytes long.
        buff.copy_from_slice(&self.data[..4]);
        FourCC { data: buff }
    }

    /// Returns the payload length.
    pub fn payload_len(&self) -> u32 {
        let mut buff: [u8; 4] = [0; 4];
        // SAFETY: Any creation of `ChunkRam` must occur through `ChunkRam::from_raw_u8`.
        // In there, we should already checked that the `data[pos..].len()` is _at least_ 8 bytes long.
        buff.copy_from_slice(&self.data[4..8]);
        u32::from_le_bytes(buff)
    }

    /// Creates a `ChunkRam` from raw array of bytes.
    pub fn from_raw_u8(data: &[u8]) -> RiffResult<ChunkRam> {
        if data.len() >= 8 {
            let chunk = ChunkRam { data: &data };
            // Guarantee that the data given is able to satisfy the payload length provided.
            if data.len() == chunk.payload_len() as usize + 8 {
                Ok(chunk)
            } else {
                Err(RiffError::PayloadLenMismatch(PayloadLenMismatch {
                    data: Vec::from(data),
                }))
            }
        } else {
            Err(RiffError::ChunkTooSmall(ChunkTooSmall {
                data: Vec::from(data),
            }))
        }
    }

    /// Returns the chunk type of this `ChunkRam` if possible.
    /// It will return an error if the data contained in this `ChunkRam` is too short to contain any.
    /// Note that this library does not guarantee if the returned `FourCC` is a valid RIFF identifier.
    pub fn chunk_type(&self) -> RiffResult<FourCC> {
        if self.data.len() >= 12 {
            let mut buff: [u8; 4] = [0; 4];
            buff.copy_from_slice(&self.data[8..12]);
            Ok(FourCC { data: buff })
        } else {
            Err(RiffError::ChunkTooSmallForChunkType(
                ChunkTooSmallForChunkType {
                    data: Vec::from(self.data),
                },
            ))
        }
    }

    /// Returns the data that this `ChunkRam` hold as raw array of bytes.
    pub fn get_raw_child(&self) -> RiffResult<&'a [u8]> {
        let offset = match self.id().as_str() {
            Ok(RIFF_ID) | Ok(LIST_ID) => 12,
            _ => 8,
        };
        // NOTE: It ought to be possible to completely avoid this check because we control the
        // creation of this struct.
        if self.data.len() >= offset {
            Ok(&self.data[offset..offset + self.payload_len() as usize])
        } else {
            Err(RiffError::PayloadLenMismatch(PayloadLenMismatch {
                data: Vec::from(self.data),
            }))
        }
    }

    /// Returns an iterator over the data of this `ChunkRam`.
    pub fn iter(&self) -> ChunkRamIter<'a> {
        match self.id().as_str() {
            Ok(RIFF_ID) | Ok(LIST_ID) => ChunkRamIter {
                cursor: 0,
                data: &self.data[12..],
                error_occurred: false,
            },
            _ => ChunkRamIter {
                cursor: 0,
                data: &self.data[8..],
                error_occurred: false,
            },
        }
    }
}

fn payload_length(data: &[u8]) -> RiffResult<u32> {
    if data.len() >= 8 {
        let mut buff: [u8; 4] = [0; 4];
        // SAFETY: Any creation of `ChunkRam` must occur through `ChunkRam::from_raw_u8`.
        // In there, we should already checked that the `data[pos..].len()` is _at least_ 8 bytes long.
        buff.copy_from_slice(&data[4..8]);
        Ok(u32::from_le_bytes(buff))
    } else {
        // Should probably be an error specific to the data begin too small.
        Err(RiffError::ChunkTooSmall(ChunkTooSmall {
            data: Vec::from(data),
        }))
    }
}

/// An iterator over the children of a `ChunkRam`.
#[derive(Debug)]
pub struct ChunkRamIter<'a> {
    cursor: u32,
    data: &'a [u8],
    error_occurred: bool,
}

/// Implementation of `ChunkRamIter`.
impl<'a> Iterator for ChunkRamIter<'a> {
    type Item = RiffResult<ChunkRam<'a>>;

    /// Get the next `ChunkRam` if it exists.
    fn next(&mut self) -> Option<Self::Item> {
        if self.error_occurred || self.cursor as usize == self.data.len() {
            None
        } else {
            let cursor = self.cursor as usize;
            match payload_length(&self.data[cursor..]) {
                Ok(payload_len) => {
                    let payload_len = payload_len as usize;
                    if self.data.len() >= cursor + 8 + payload_len {
                        match ChunkRam::from_raw_u8(&self.data[cursor..cursor + 8 + payload_len]) {
                            Ok(chunk) => {
                                self.cursor = self.cursor
                                    + 8
                                    + chunk.payload_len()
                                    + (chunk.payload_len() % 2);
                                Some(Ok(chunk))
                            }
                            Err(err) => {
                                self.error_occurred = true;
                                Some(Err(err))
                            }
                        }
                    } else {
                        self.error_occurred = true;
                        Some(Err(RiffError::ChunkTooSmall(ChunkTooSmall {
                            data: Vec::from(&self.data[cursor..]),
                        })))
                    }
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
    /// Represents a `ChunkRam` where the payload contains the chunk's identifier, the chunk's type and a list of `ChunkRamContent`s.
    Children(FourCC, FourCC, Vec<ChunkRamContent<'a>>),
    /// Represents a `ChunkRam` where the payload only contain a list of `ChunkRamContent`s.
    ChildrenNoType(FourCC, Vec<ChunkRamContent<'a>>),
}

/// Since `ChunkRam` is an opaque type. The only way to obtain the `ChunkRam`'s contents is through this trait.
impl<'a> TryFrom<ChunkRam<'a>> for ChunkRamContent<'a> {
    type Error = RiffError;

    /// Performs the conversion.
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
