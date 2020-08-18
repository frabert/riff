use std::fmt::Formatter;

/// The type of errors that this library may emit.
/// Note that most of this errors are currently unused.
/// There are many, many ways reading into a RIFF file may fail.
#[derive(Debug)]
pub enum RiffError {
    /// IO errors that will be emitted by standard IO.
    Io(std::io::Error),
    /// Indicates that the given file is not large enough to be a valid RIFF file.
    /// RIFF files must be at least 8 bytes long to contain the 4 bytes ASCII identifier and the 4 bytes little-endian `u32`.
    ByteLessThan8(usize),
    /// Indicates that the provided payload length does not match the raw data's length.
    /// Since the data may be a list of `Chunk`s, it is more likely that this error is caused when payload's length > raw data's size.
    PayloadLenMismatch(usize, u32),
    Utf8Error(std::str::Utf8Error),
    NoneError,
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
    fn from(_: std::option::NoneError) -> Self {
        RiffError::NoneError
    }
}

/// A convenient `Result` type.
pub type RiffResult<T> = Result<T, RiffError>;
