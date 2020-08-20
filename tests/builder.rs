extern crate riffu;

use riffu::builder::riff::{Chunk, ChunkChildren, Riff};
use riffu::error::RiffResult;
use riffu::FourCC;

#[test]
pub fn creation() -> RiffResult<()> {
    let builder = Riff::new(FourCC::new("smpl")?).add_chunk(Chunk::new_notype(
        FourCC::new("test")?,
        ChunkChildren::RawData(vec![255]),
    ));
    println!("builder:{:?}", builder.to_bytes());
    Ok(())
}
