use crate::error::RiffError;
use std::convert::{TryFrom, TryInto};

pub mod builder;
pub mod constants;
pub mod eager;
pub mod error;
pub mod lazy;

#[derive(Debug, Clone)]
pub struct FourCC {
    data: [u8; 4],
}

/// Represents the ASCII identifier of a chunk.
///
/// # Example
///
/// ```rust
/// use riffu::FourCC;
/// let good = FourCC::new("1234");
/// assert!(good.is_ok());
/// let bad = FourCC::new("12345");
/// assert!(bad.is_err());
/// ```
///
/// # NOTE
///
/// 1. AFAIK, the only valid identifier is in ASCII.
///    I am not entirely sure what all the possible value ASCII can take.
///    So we need to enforce correctness on that front too.
/// 2. Are there other conversions that we are missing out on?
impl FourCC {
    pub fn new(data: &[u8; 4]) -> Self {
        FourCC { data: *data }
    }

    /// View `&self` struct as a `&[u8]`.
    pub fn as_bytes(&self) -> &[u8; 4] {
        &self.data
    }

    /// Consume `self` and returns a `[u8; 4]`.
    pub fn into_bytes(self) -> [u8; 4] {
        self.data
    }
}

/// A `&[u8]` can be converted to a `FourCC`.
impl TryFrom<&[u8]> for FourCC {
    type Error = RiffError;

    /// Performs the conversion.
    /// ```
    /// use riffu::FourCC;
    /// use std::convert::TryInto;
    /// let buffer: &[u8] = &[80u8, 80u8, 80u8,80u8];
    /// let test: FourCC = buffer.try_into().unwrap();
    /// ```
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(FourCC {
            data: value.try_into()?,
        })
    }
}

/// A `&str` can be converted to a `FourCC`.
impl TryFrom<&str> for FourCC {
    type Error = RiffError;

    /// Performs the conversion.
    /// ```
    /// use riffu::FourCC;
    /// use std::convert::TryInto;
    /// let test : FourCC = "test".try_into().unwrap();
    /// ```
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(value.as_bytes().try_into()?)
    }
}
