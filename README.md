# riff

## Crate for doing IO on RIFF-formatted files

This crate provides utility methods for reading and writing formats such as
Microsoft Wave, Audio Video Interleave or Downloadable Sounds.

### Examples

Reading chunks:

````rust
let mut file = File::open("somefile.wav")?;
let chunk = riff::Chunk::read(&mut file, 0)?;

for child in chunk.iter(&mut file) {
  println!(child.id());
}
````

Writing chunks:

````rust
// Some ids to use while creating chunks
let smpl_id: ChunkId = ChunkId { value: [0x73, 0x6D, 0x70, 0x6C] };
let test_id: ChunkId = ChunkId { value: [0x74, 0x65, 0x73, 0x74] };
let tst1_id: ChunkId = ChunkId { value: [0x74, 0x73, 0x74, 0x31] };
let tst2_id: ChunkId = ChunkId { value: [0x74, 0x73, 0x74, 0x32] };

let str1 = "hey this is a test".as_bytes().to_vec();
let str2 = "hey this is another test".as_bytes().to_vec();
let str3 = "final test".as_bytes().to_vec();

let contents = ChunkContents::Children(
  riff::RIFF_ID,
  smpl_id,
  vec![
    ChunkContents::Children(
      riff::LIST_ID,
      tst1_id,
      vec![
        ChunkContents::Data(test_id, str1),
        ChunkContents::Data(test_id, str2)
      ]),
    ChunkContents::Children(
      riff::LIST_ID,
      tst2_id,
      vec![
        ChunkContents::Data(test_id, str3)
      ]
    )
  ]);

let mut file = File::create("somefile.riff")?;
contents.write(&mut file)?;
````
