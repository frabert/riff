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
}

/// Any `std::io::Error` will represent `RiffError::Io`
impl From<std::io::Error> for RiffError {
    fn from(v: std::io::Error) -> Self {
        RiffError::Io(v)
    }
}
