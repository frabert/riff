#![feature(try_trait)]

use crate::error::{RiffError, RiffResult};

pub mod builder;
pub mod constants;
pub mod eager;
pub mod error;
pub mod lazy;

#[derive(Debug, Clone)]
pub struct ChunkId {
    data: [u8; 4],
}

impl ChunkId {
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

#[derive(Debug, Clone)]
pub struct ChunkType {
    data: [u8; 4],
}

impl ChunkType {
    pub fn as_str(&self) -> RiffResult<&str> {
        Ok(std::str::from_utf8(&self.data)?)
    }

    pub fn as_bytes(&self) -> &[u8; 4] {
        &self.data
    }

    pub fn into_bytes(self) -> [u8; 4] {
        self.data
    }
}
