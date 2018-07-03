# riff

## Crate for doing IO on RIFF-formatted files

This crate provides utility methods for reading and writing formats such as
Microsoft Wave, Audio Video Interleave or Downloadable Sounds.

### Examples

Reading chunks:

````rust
let mut file = File::open("somefile.wav")?;
let (chunk, _len) = riff::read_chunk(&mut file)?;
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

let test_1 = Chunk::new_data(test_id.clone(), str1);
let test_2 = Chunk::new_data(test_id.clone(), str2);
let test_3 = Chunk::new_data(test_id.clone(), str3);

let tst1 = Chunk::new_list(tst1_id, vec![test_1, test_2]);
let tst2 = Chunk::new_list(tst2_id, vec![test_3]);

let chunk = Chunk::new_riff(smpl_id, vec![tst1, tst2]);

let mut file = File::create("somefile.riff")?;
riff::write_chunk(&mut file, &chunk);
````