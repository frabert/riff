extern crate riff;

use riff::Chunk;
use riff::ChunkContents;
use riff::ChunkId;

use std::fs::File;

static SMPL_ID: ChunkId = ChunkId {
    value: [0x73, 0x6D, 0x70, 0x6C],
};
static TEST_ID: ChunkId = ChunkId {
    value: [0x74, 0x65, 0x73, 0x74],
};
static TEST_1_ID: ChunkId = ChunkId {
    value: [0x74, 0x73, 0x74, 0x31],
};
static TEST_2_ID: ChunkId = ChunkId {
    value: [0x74, 0x73, 0x74, 0x32],
};

#[test]
fn read_as_riff() {
    let file = riff::Riff::from_file(std::path::PathBuf::from("test_assets/minimal.riff")).unwrap();
    println!("file:{:?}", file);
    let _ = file
        .get_chunk()
        .child_iter()
        .inspect(|chunk| {
            println!("chunk:{:?}", chunk);
            println!("chunk_id:{:?}", chunk.id().as_str());
        })
        .map(ChunkContents::from)
        .inspect(|content| println!("content:{:?}", content))
        .collect::<Vec<ChunkContents>>();
}
