use std::fmt::Formatter;

/// The type of errors that this library may emit.
/// Note that most of this errors are currently unused.
/// There are many, many ways reading into a RIFF file may fail.
#[derive(Debug)]
pub enum RiffError {
    /// IO errors that will be emitted by standard IO.
    Io(std::io::Error),
    /// Indicates that the provided payload length does not match the raw data's length.
    /// Since the data may be a list of `Chunk`s, it is more likely that this error is caused when payload's length > raw data's size.
    PayloadLenMismatch(PayloadLenMismatch),
    /// Indicates that the requested data is too small to be a valid chunk.
    /// Note that this returns the entire data and the starting position where this "chunk" is supposed to reside.
    ChunkTooSmall(ChunkTooSmall),
    /// Indicates that the `Chunk` is too small to contain a `FourCC`.I
    ChunkTooSmallForChunkType(ChunkTooSmallForChunkType),
    Utf8Error(std::str::Utf8Error),
    /// Indicates that this is a malformed RIFF file.
    /// RIFF file requires that the first 4 bytes of the file contains the ASCII letters "RIFF".
    InvalidRiffHeader,
    /// Indicates a `None` error caused by unwrapping a `None`.
    NoneError(std::option::NoneError),
    /// Indicates a failure when trying to convert from `&str` to a slice.
    TryFromSliceError(std::array::TryFromSliceError),
}

#[derive(Debug)]
pub struct ChunkTooSmallForChunkType {
    pub(crate) data: Vec<u8>,
    pub(crate) pos: usize,
}

impl std::fmt::Display for ChunkTooSmallForChunkType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

#[derive(Debug)]
pub struct ChunkTooSmall {
    pub(crate) data: Vec<u8>,
    pub(crate) pos: usize,
}

impl std::fmt::Display for ChunkTooSmall {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

#[derive(Debug)]
pub struct PayloadLenMismatch {
    pub(crate) data: Vec<u8>,
    pub(crate) pos: usize,
    pub(crate) offset: usize,
    pub(crate) payload_len: usize,
}

impl std::fmt::Display for PayloadLenMismatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl std::error::Error for RiffError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self)
    }
}

impl std::fmt::Display for RiffError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

/// Converts `std::io::Error`.
impl From<std::io::Error> for RiffError {
    /// Performs the conversion.
    fn from(v: std::io::Error) -> Self {
        RiffError::Io(v)
    }
}

/// Converts `std::str::Utf8Error`.
impl From<std::str::Utf8Error> for RiffError {
    /// Performs the conversion.
    fn from(v: std::str::Utf8Error) -> Self {
        RiffError::Utf8Error(v)
    }
}

/// Converts `std::option::NoneError`.
impl From<std::option::NoneError> for RiffError {
    /// Performs the conversion.
    fn from(v: std::option::NoneError) -> Self {
        RiffError::NoneError(v)
    }
}

/// Converts `std::option::NoneError`.
impl From<std::array::TryFromSliceError> for RiffError {
    /// Performs the conversion.
    fn from(v: std::array::TryFromSliceError) -> Self {
        RiffError::TryFromSliceError(v)
    }
}

/// A convenient `Result` type.
pub type RiffResult<T> = Result<T, RiffError>;
