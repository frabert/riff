extern crate riffu;

use riffu::{
    constants::{LIST_ID, RIFF_ID},
    lazy::riff::ChunkDisk,
};

#[test]
fn test_set_1() {
    let mut chunk_root = ChunkDisk::from_path("test_assets/set_1.riff").unwrap();
    assert_eq!(chunk_root.payload_len().unwrap(), 14);
    assert_eq!(chunk_root.id().unwrap().as_bytes(), b"RIFF");
    assert_eq!(chunk_root.chunk_type().unwrap().as_bytes(), b"smpl");
    let expected_content = vec![("test", vec![255])];
    assert_eq!(
        chunk_root.iter().unwrap().fold(0, |acc, _| acc + 1),
        expected_content.len()
    );
    for (chunk, (expected_name, expected_data)) in chunk_root.iter().unwrap().zip(expected_content)
    {
        let mut chunk = chunk.unwrap();
        assert_eq!("test", expected_name);
        assert_eq!(chunk.get_raw_child().unwrap().len(), expected_data.len());
        assert_eq!(chunk.get_raw_child().unwrap(), expected_data);
    }
}

#[test]
fn test_set_2() {
    let mut chunk_root = ChunkDisk::from_path("test_assets/set_2.riff").unwrap();
    assert_eq!(chunk_root.payload_len().unwrap(), 24);
    assert_eq!(chunk_root.id().unwrap().as_bytes(), b"RIFF");
    assert_eq!(chunk_root.chunk_type().unwrap().as_bytes(), b"smpl");
    let expected_content = vec![(b"tst1", vec![255]), (b"tst2", vec![238])];
    assert_eq!(
        chunk_root.iter().unwrap().fold(0, |acc, _| acc + 1),
        expected_content.len()
    );
    for (chunk, (name, data)) in chunk_root.iter().unwrap().zip(expected_content) {
        let mut chunk = chunk.unwrap();
        assert_eq!(chunk.id().unwrap().as_bytes(), name);
        assert_eq!(chunk.get_raw_child().unwrap().len(), data.len());
        assert_eq!(chunk.get_raw_child().unwrap(), data);
    }
}

#[test]
fn test_set_3() {
    let mut chunk_root = ChunkDisk::from_path("test_assets/set_3.riff").unwrap();
    {
        assert_eq!(chunk_root.payload_len().unwrap(), 100);
        assert_eq!(
            chunk_root.id().unwrap().as_bytes(),
            riffu::constants::RIFF_ID
        );
        assert_eq!(chunk_root.chunk_type().unwrap().as_bytes(), b"smpl");
        assert_eq!(chunk_root.iter().unwrap().fold(0, |acc, _| acc + 1), 2);
    }
    {
        let mut list_1 = chunk_root.iter().unwrap().next().unwrap().unwrap();
        assert_eq!(list_1.id().unwrap().as_bytes(), riffu::constants::LIST_ID);
        assert_eq!(list_1.chunk_type().unwrap().as_bytes(), b"tst1");
        assert_eq!(list_1.iter().unwrap().fold(0, |acc, _| acc + 1), 2);
        {
            let mut test = list_1.iter().unwrap().next().unwrap().unwrap();
            assert_eq!(test.id().unwrap().as_bytes(), b"test");
            assert_eq!(
                test.get_raw_child().unwrap(),
                "hey this is a test".as_bytes()
            );
        }
        {
            let mut test = list_1.iter().unwrap().skip(1).next().unwrap().unwrap();
            assert_eq!(test.id().unwrap().as_bytes(), b"test");
            assert_eq!(
                test.get_raw_child().unwrap(),
                "hey this is another test".as_bytes()
            );
        }
    }
    {
        let mut list_1 = chunk_root.iter().unwrap().skip(1).next().unwrap().unwrap();
        assert_eq!(list_1.id().unwrap().as_bytes(), b"seqt");
        assert_eq!(list_1.iter().unwrap().fold(0, |acc, _| acc + 1), 1);
        assert_eq!(
            list_1
                .iter()
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .id()
                .unwrap()
                .as_bytes(),
            b"test"
        );
        assert_eq!(
            list_1
                .iter()
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .get_raw_child()
                .unwrap(),
            b"final test"
        );
    }
}

#[test]
fn test_set_4() {
    let mut chunk_root = ChunkDisk::from_path("test_assets/set_4.riff").unwrap();
    {
        assert_eq!(chunk_root.payload_len().unwrap(), 102);
        assert_eq!(chunk_root.id().unwrap().as_bytes(), RIFF_ID);
        assert_eq!(chunk_root.chunk_type().unwrap().as_bytes(), b"smpl");
        assert_eq!(chunk_root.iter().unwrap().fold(0, |acc, _| acc + 1), 2);
    }
    {
        let mut list_1 = chunk_root.iter().unwrap().next().unwrap().unwrap();
        assert_eq!(list_1.id().unwrap().as_bytes(), LIST_ID);
        assert_eq!(list_1.chunk_type().unwrap().as_bytes(), b"tst1");
        assert_eq!(list_1.iter().unwrap().fold(0, |acc, _| acc + 1), 2);
        {
            let mut test = list_1.iter().unwrap().next().unwrap().unwrap();
            assert_eq!(test.id().unwrap().as_bytes(), b"test");
            assert_eq!(
                test.get_raw_child().unwrap(),
                "hey this is a test".as_bytes()
            );
        }
        {
            let mut test = list_1.iter().unwrap().skip(1).next().unwrap().unwrap();
            assert_eq!(test.id().unwrap().as_bytes(), b"test");
            assert_eq!(
                test.get_raw_child().unwrap(),
                "hey this is another test!".as_bytes()
            );
        }
    }
    {
        let mut list_1 = chunk_root.iter().unwrap().skip(1).next().unwrap().unwrap();
        assert_eq!(list_1.id().unwrap().as_bytes(), b"seqt");
        assert_eq!(list_1.iter().unwrap().fold(0, |acc, _| acc + 1), 1);
        assert_eq!(
            list_1
                .iter()
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .id()
                .unwrap()
                .as_bytes(),
            b"test"
        );
        assert_eq!(
            list_1
                .iter()
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .get_raw_child()
                .unwrap(),
            b"final test"
        );
    }
}
