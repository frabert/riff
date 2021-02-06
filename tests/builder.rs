extern crate riffu;

use riffu::constants::LIST_ID;
use riffu::{
    builder::riff::{ChunkBuilder, ChunkData, RiffBuilder},
    eager::riff::RiffRam,
    FourCC,
};

#[test]
pub fn test_size() {
    let r = b"smpl";
    assert_eq!(RiffBuilder::new(FourCC::new(r)).payload_len, 4);
    let chunk_1 = ChunkBuilder::new_type(
        FourCC::new(b"test"),
        FourCC::new(b"test"),
        ChunkData::RawData(vec![]),
    );
    assert_eq!(chunk_1.payload_len, 4);
    let chunk_2 = ChunkBuilder::new_notype(FourCC::new(b"test"), ChunkData::RawData(vec![]));
    assert_eq!(chunk_2.payload_len, 0);
    let built_riff = RiffBuilder::new(FourCC::new(b"smpl"))
        .add_chunk(chunk_1)
        .add_chunk(chunk_2);
    assert_eq!(built_riff.payload_len, 4 + (4 + 4 + 4) + (4 + 4));
}

#[test]
pub fn test_set_1() {
    let read_riff = RiffRam::from_file("test_assets/set_1.riff").unwrap();
    let built_riff = RiffBuilder::new(FourCC::new(b"smpl")).add_chunk(ChunkBuilder::new_notype(
        FourCC::new(b"test"),
        ChunkData::RawData(vec![255]),
    ));
    assert_eq!(read_riff.as_bytes(), built_riff.to_bytes());
}

#[test]
pub fn test_set_2() {
    let read_riff = RiffRam::from_file("test_assets/set_2.riff").unwrap();
    let built_riff = RiffBuilder::new(FourCC::new(b"smpl"))
        .add_chunk(ChunkBuilder::new_notype(
            FourCC::new(b"tst1"),
            ChunkData::RawData(vec![255]),
        ))
        .add_chunk(ChunkBuilder::new_notype(
            FourCC::new(b"tst2"),
            ChunkData::RawData(vec![238]),
        ));
    assert_eq!(read_riff.as_bytes(), built_riff.to_bytes());
}

#[test]
pub fn test_set_3() {
    let read_riff = RiffRam::from_file("test_assets/set_3.riff").unwrap();
    let built_riff = RiffBuilder::new(FourCC::new(b"smpl"))
        .add_chunk(ChunkBuilder::new_type(
            FourCC::new(LIST_ID),
            FourCC::new(b"tst1"),
            ChunkData::ChunkList(vec![
                ChunkBuilder::new_notype(
                    FourCC::new(b"test"),
                    ChunkData::RawData("hey this is a test".into()),
                ),
                ChunkBuilder::new_notype(
                    FourCC::new(b"test"),
                    ChunkData::RawData("hey this is another test".into()),
                ),
            ]),
        ))
        .add_chunk(ChunkBuilder::new_notype(
            FourCC::new(b"seqt"),
            ChunkData::ChunkList(vec![ChunkBuilder::new_notype(
                FourCC::new(b"test"),
                ChunkData::RawData("final test".into()),
            )]),
        ));
    assert_eq!(read_riff.as_bytes(), built_riff.to_bytes());
}
