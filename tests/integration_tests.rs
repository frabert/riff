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
static TST1_ID: ChunkId = ChunkId {
    value: [0x74, 0x73, 0x74, 0x31],
};
static TST2_ID: ChunkId = ChunkId {
    value: [0x74, 0x73, 0x74, 0x32],
};

fn read_items<T>(iter: &mut riff::Iter<T>) -> Vec<Chunk>
where
    T: std::io::Read + std::io::Seek,
{
    let mut vec: Vec<Chunk> = Vec::new();
    for item in iter {
        match item {
            Ok(chunk) => vec.push(chunk),
            _ => (),
        }
    }
    vec
}

#[test]
fn read_minimal() {
    let mut file = File::open("test_assets/minimal.riff").unwrap();
    let chunk = riff::Chunk::read(&mut file, 0).unwrap();

    assert_eq!(chunk.id(), riff::RIFF_ID);
    assert_eq!(chunk.read_type(&mut file).unwrap(), SMPL_ID);

    let items = read_items(&mut chunk.iter(&mut file));

    assert_eq!(items.len(), 1);

    let item = &items[0];

    assert_eq!(item.id(), TEST_ID);
    assert_eq!(item.read_contents(&mut file).unwrap(), vec![0xFF]);
}

#[test]
fn read_minimal2() {
    let mut file = File::open("test_assets/minimal_2.riff").unwrap();
    let chunk = riff::Chunk::read(&mut file, 0).unwrap();

    assert_eq!(chunk.id(), riff::RIFF_ID);
    assert_eq!(chunk.read_type(&mut file).unwrap(), SMPL_ID);

    let items = read_items(&mut chunk.iter(&mut file));

    assert_eq!(items.len(), 2);

    let item1 = &items[0];
    let item2 = &items[1];

    assert_eq!(item1.id(), TST1_ID);
    assert_eq!(item1.read_contents(&mut file).unwrap(), vec![0xFF]);

    assert_eq!(item2.id(), TST2_ID);
    assert_eq!(item2.read_contents(&mut file).unwrap(), vec![0xEE]);
}

#[test]
fn read_test_1() {
    let mut file = File::open("test_assets/test.riff").unwrap();
    let str1 = "hey this is a test".as_bytes().to_vec();
    let str2 = "hey this is another test".as_bytes().to_vec();
    let str3 = "final test".as_bytes().to_vec();

    let smpl = Chunk::read(&mut file, 0).unwrap();
    assert_eq!(smpl.id(), riff::RIFF_ID);
    assert_eq!(smpl.read_type(&mut file).unwrap(), SMPL_ID);

    let smpl_items = read_items(&mut smpl.iter(&mut file));
    assert_eq!(smpl_items.len(), 2);

    let tst1 = &smpl_items[0];
    assert_eq!(tst1.id(), riff::LIST_ID);
    assert_eq!(tst1.read_type(&mut file).unwrap(), TST1_ID);
    let tst1_items = read_items(&mut tst1.iter(&mut file));
    assert_eq!(tst1_items.len(), 2);

    let test_1 = &tst1_items[0];
    let test_2 = &tst1_items[1];

    assert_eq!(test_1.id(), TEST_ID);
    assert_eq!(test_1.read_contents(&mut file).unwrap(), str1);

    assert_eq!(test_2.id(), TEST_ID);
    assert_eq!(test_2.read_contents(&mut file).unwrap(), str2);

    let tst2 = &smpl_items[1];
    assert_eq!(tst2.id(), riff::SEQT_ID);

    let tst2_items = read_items(&mut tst2.iter_no_type(&mut file));
    assert_eq!(tst2_items.len(), 1);

    let test_3 = &tst2_items[0];
    assert_eq!(test_3.id(), TEST_ID);
    assert_eq!(test_3.read_contents(&mut file).unwrap(), str3);
}

#[test]
fn read_test_2() {
    let mut file = File::open("test_assets/test_2.riff").unwrap();
    let str1 = "hey this is a test".as_bytes().to_vec();
    let str2 = "hey this is another test!".as_bytes().to_vec();
    let str3 = "final test".as_bytes().to_vec();

    let smpl = Chunk::read(&mut file, 0).unwrap();
    assert_eq!(smpl.id(), riff::RIFF_ID);
    assert_eq!(smpl.read_type(&mut file).unwrap(), SMPL_ID);

    let smpl_items = read_items(&mut smpl.iter(&mut file));
    assert_eq!(smpl_items.len(), 2);

    let tst1 = &smpl_items[0];
    assert_eq!(tst1.id(), riff::LIST_ID);
    assert_eq!(tst1.read_type(&mut file).unwrap(), TST1_ID);
    let tst1_items = read_items(&mut tst1.iter(&mut file));
    assert_eq!(tst1_items.len(), 2);

    let test_1 = &tst1_items[0];
    let test_2 = &tst1_items[1];

    assert_eq!(test_1.id(), TEST_ID);
    assert_eq!(test_1.read_contents(&mut file).unwrap(), str1);

    assert_eq!(test_2.id(), TEST_ID);
    assert_eq!(test_2.read_contents(&mut file).unwrap(), str2);

    let tst2 = &smpl_items[1];
    assert_eq!(tst2.id(), riff::SEQT_ID);

    let tst2_items = read_items(&mut tst2.iter_no_type(&mut file));
    assert_eq!(tst2_items.len(), 1);

    let test_3 = &tst2_items[0];
    assert_eq!(test_3.id(), TEST_ID);
    assert_eq!(test_3.read_contents(&mut file).unwrap(), str3);
}

fn read_chunk<T>(chunk: &Chunk, file: &mut T) -> ChunkContents
where
    T: std::io::Seek + std::io::Read,
{
    let id = chunk.id();
    if id == riff::RIFF_ID || id == riff::LIST_ID {
        let chunk_type = chunk.read_type(file).unwrap();
        let children = read_items(&mut chunk.iter(file));
        let mut children_contents: Vec<ChunkContents> = Vec::new();

        for child in children {
            children_contents.push(read_chunk(&child, file));
        }

        ChunkContents::Children(id, chunk_type, children_contents)
    } else if id == riff::SEQT_ID {
        let children = read_items(&mut chunk.iter_no_type(file));
        let mut children_contents: Vec<ChunkContents> = Vec::new();

        for child in children {
            children_contents.push(read_chunk(&child, file));
        }

        ChunkContents::ChildrenNoType(id, children_contents)
    } else {
        let contents = chunk.read_contents(file).unwrap();
        ChunkContents::Data(id, contents)
    }
}

#[test]
fn write_test_1() {
    let buf: Vec<u8> = vec![0; 1024];
    let chunk = {
        let mut file_read = File::open("test_assets/test.riff").unwrap();
        let chunk = Chunk::read(&mut file_read, 0).unwrap();
        read_chunk(&chunk, &mut file_read)
    };
    let mut cursor = std::io::Cursor::new(buf);
    chunk.write(&mut cursor).unwrap();

    let reread_chunk = {
        let chunk = Chunk::read(&mut cursor, 0).unwrap();
        read_chunk(&chunk, &mut cursor)
    };

    assert_eq!(chunk, reread_chunk);
}

#[test]
fn write_test_2() {
    let buf: Vec<u8> = vec![0; 1024];
    let chunk = {
        let mut file_read = File::open("test_assets/test_2.riff").unwrap();
        let chunk = Chunk::read(&mut file_read, 0).unwrap();
        read_chunk(&chunk, &mut file_read)
    };
    let mut cursor = std::io::Cursor::new(buf);
    chunk.write(&mut cursor).unwrap();

    let reread_chunk = {
        let chunk = Chunk::read(&mut cursor, 0).unwrap();
        read_chunk(&chunk, &mut cursor)
    };

    assert_eq!(chunk, reread_chunk);
}
