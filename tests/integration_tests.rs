extern crate riff;

use riff::ChunkContents;

#[test]
fn read_as_riff() {
    let file = riff::Riff::from_file(std::path::PathBuf::from("test_assets/minimal.riff")).unwrap();
    println!("file:{:?} file.data.len():{:?}", file, file.len());
    let root_chunk = file.get_chunk();
    println!(
        "root_chunk:{:?}\nroot_chunk_id:{:?}",
        root_chunk,
        root_chunk.id().as_str()
    );
    root_chunk
        .iter_type()
        .inspect(|chunk| {
            println!("chunk:{:?}\nchunk_id:{:?}", chunk, chunk.id().as_str());
        })
        .map(ChunkContents::from)
        .for_each(|content| println!("content:{:?}", content));
}
