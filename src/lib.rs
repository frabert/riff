#![feature(try_trait)]
#![feature(iterator_fold_self)]

use crate::error::{RiffError, RiffResult};
use std::convert::TryInto;

pub mod builder;
pub mod constants;
pub mod eager;
pub mod error;
pub mod lazy;

#[derive(Debug, Clone)]
pub struct FourCC {
    data: [u8; 4],
}

impl FourCC {
    pub fn new(input: &str) -> RiffResult<Self> {
        Ok(FourCC {
            data: input.as_bytes().try_into()?,
        })
    }

    pub fn as_str(&self) -> Result<&str, RiffError> {
        Ok(std::str::from_utf8(&self.data)?)
    }

    pub fn as_bytes(&self) -> &[u8; 4] {
        &self.data
    }

    pub fn into_bytes(self) -> [u8; 4] {
        self.data
    }
}
