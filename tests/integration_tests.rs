extern crate riff;

use riff::Chunk;
use riff::ChunkId;

use std::fs::File;

static SMPL_ID: ChunkId = ChunkId { value: [0x73, 0x6D, 0x70, 0x6C] };
static TEST_ID: ChunkId = ChunkId { value: [0x74, 0x65, 0x73, 0x74] };
static TST1_ID: ChunkId = ChunkId { value: [0x74, 0x73, 0x74, 0x31] };
static TST2_ID: ChunkId = ChunkId { value: [0x74, 0x73, 0x74, 0x32] };

#[test]
fn read_minimal() {
  let mut file = File::open("test_assets/minimal.riff").unwrap();
  let (chunk, _len) = riff::read_chunk(&mut file).unwrap();
  let expected = Chunk::new_riff(SMPL_ID.clone(),
    vec![
      Chunk::new_data(TEST_ID.clone(), vec![0xFF])
    ]
  );
  assert_eq!(chunk, expected);
}

#[test]
fn read_minimal2() {
  let mut file = File::open("test_assets/minimal_2.riff").unwrap();
  let (chunk, _len) = riff::read_chunk(&mut file).unwrap();
  let expected = Chunk::new_riff(SMPL_ID.clone(), vec![
      Chunk::new_data(TST1_ID.clone(), vec![0xFF]),
      Chunk::new_data(TST2_ID.clone(), vec![0xEE])
    ]
  );
  assert_eq!(chunk, expected);
}

#[test]
fn read_test_1() {
  let mut file = File::open("test_assets/test.riff").unwrap();
  let str1 = "hey this is a test".as_bytes().to_vec();
  let str2 = "hey this is another test".as_bytes().to_vec();
  let str3 = "final test".as_bytes().to_vec();

  let test_1 = Chunk::new_data(TEST_ID.clone(), str1);
  let test_2 = Chunk::new_data(TEST_ID.clone(), str2);
  let test_3 = Chunk::new_data(TEST_ID.clone(), str3);

  let tst1 = Chunk::new_list(TST1_ID.clone(), vec![test_1, test_2]);
  let tst2 = Chunk::new_seqt(TST2_ID.clone(), vec![test_3]);

  let (chunk, _len) = riff::read_chunk(&mut file).unwrap();
  let expected = Chunk::new_riff(SMPL_ID.clone(), vec![tst1, tst2]);
  assert_eq!(chunk, expected);
}

#[test]
fn read_test_2() {
  let mut file = File::open("test_assets/test_2.riff").unwrap();
  let str1 = "hey this is a test".as_bytes().to_vec();
  let str2 = "hey this is another test!".as_bytes().to_vec();
  let str3 = "final test".as_bytes().to_vec();

  let test_1 = Chunk::new_data(TEST_ID.clone(), str1);
  let test_2 = Chunk::new_data(TEST_ID.clone(), str2);
  let test_3 = Chunk::new_data(TEST_ID.clone(), str3);

  let tst1 = Chunk::new_list(TST1_ID.clone(), vec![test_1, test_2]);
  let tst2 = Chunk::new_seqt(TST2_ID.clone(), vec![test_3]);

  let (chunk, _len) = riff::read_chunk(&mut file).unwrap();
  let expected = Chunk::new_riff(SMPL_ID.clone(), vec![tst1, tst2]);
  assert_eq!(chunk, expected);
}

#[test]
fn write_test_1() {
  let read_chunk : Chunk;
  let mut buf: Vec<u8> = Vec::new();

  {
    let mut file_read = File::open("test_assets/test.riff").unwrap();
    let (chunk, _len) = riff::read_chunk(&mut file_read).unwrap();
    read_chunk = chunk;
  }
  
  riff::write_chunk(&mut buf, &read_chunk).unwrap();
  
  let mut cursor = std::io::Cursor::new(buf);
  let (chunk, _len) = riff::read_chunk(&mut cursor).unwrap();
  assert_eq!(chunk, read_chunk);
}

#[test]
fn write_test_2() {
  let read_chunk : Chunk;
  let mut buf: Vec<u8> = Vec::new();

  {
    let mut file_read = File::open("test_assets/test_2.riff").unwrap();
    let (chunk, _len) = riff::read_chunk(&mut file_read).unwrap();
    read_chunk = chunk;
  }
  
  riff::write_chunk(&mut buf, &read_chunk).unwrap();
  
  let mut cursor = std::io::Cursor::new(buf);
  let (chunk, _len) = riff::read_chunk(&mut cursor).unwrap();
  assert_eq!(chunk, read_chunk);
}