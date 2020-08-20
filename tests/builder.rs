extern crate riffu;

use riffu::{
    builder::riff::{ChunkBuilder, ChunkData, RiffBuilder},
    eager::riff::RiffRam,
    error::RiffResult,
    FourCC,
};

#[test]
pub fn test_minimal() -> RiffResult<()> {
    let read_riff = RiffRam::from_file("test_assets/set_1.riff")?;
    let built_riff = RiffBuilder::new(FourCC::new("smpl")?).add_chunk(ChunkBuilder::new_notype(
        FourCC::new("test")?,
        ChunkData::RawData(vec![255]),
    ));
    assert_eq!(read_riff.as_bytes(), built_riff.to_bytes());
    Ok(())
}

#[test]
pub fn test_minimal_2() -> RiffResult<()> {
    let read_riff = RiffRam::from_file("test_assets/set_2.riff")?;
    let built_riff = RiffBuilder::new(FourCC::new("smpl")?)
        .add_chunk(ChunkBuilder::new_notype(
            FourCC::new("tst1")?,
            ChunkData::RawData(vec![255]),
        ))
        .add_chunk(ChunkBuilder::new_notype(
            FourCC::new("tst2")?,
            ChunkData::RawData(vec![238]),
        ));
    assert_eq!(read_riff.as_bytes(), built_riff.to_bytes());
    Ok(())
}
