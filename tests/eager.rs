extern crate riffu;

use riffu::{
    eager::riff::{ChunkRam, ChunkRamIter, RiffRam},
    error::RiffResult,
};
use std::convert::TryFrom;

#[test]
fn test_set_1() {
    let file = RiffRam::from_file("test_assets/set_1.riff").unwrap();
    assert_eq!(file.payload_len(), 14);
    assert_eq!(ChunkRam::try_from(&file).unwrap().id().as_bytes(), b"RIFF");
    assert_eq!(
        ChunkRam::try_from(&file)
            .unwrap()
            .chunk_type()
            .unwrap()
            .as_bytes(),
        b"smpl"
    );
    let expected_content = vec![vec![255]];
    assert_eq!(
        file.iter().unwrap().fold(0, |acc, _| acc + 1),
        expected_content.len()
    );
    for (chunk, expected) in file.iter().unwrap().zip(expected_content) {
        let chunk = chunk.unwrap();
        assert_eq!(chunk.get_raw_child().unwrap().len(), expected.len());
        assert_eq!(chunk.get_raw_child().unwrap(), expected);
    }
    match (
        file.iter().unwrap().skip(1).next(),
        None::<RiffResult<ChunkRamIter>>,
    ) {
        (None, None) => assert!(true),
        _ => assert!(false),
    }
}

#[test]
fn test_set_2() {
    let file = RiffRam::from_file("test_assets/set_2.riff").unwrap();
    assert_eq!(file.payload_len(), 24);
    assert_eq!(ChunkRam::try_from(&file).unwrap().id().as_bytes(), b"RIFF");
    assert_eq!(
        ChunkRam::try_from(&file)
            .unwrap()
            .chunk_type()
            .unwrap()
            .as_bytes(),
        b"smpl"
    );
    let expected_content = vec![(b"tst1", vec![255]), (b"tst2", vec![238])];
    assert_eq!(
        file.iter().unwrap().fold(0, |acc, _| acc + 1),
        expected_content.len()
    );
    for (chunk, (name, data)) in file.iter().unwrap().zip(expected_content) {
        let chunk = chunk.unwrap();
        assert_eq!(chunk.id().as_bytes(), name);
        assert_eq!(chunk.get_raw_child().unwrap().len(), data.len());
        assert_eq!(chunk.get_raw_child().unwrap(), data);
    }
    match (
        file.iter().unwrap().skip(2).next(),
        None::<RiffResult<ChunkRamIter>>,
    ) {
        (None, None) => assert!(true),
        _ => assert!(false),
    }
}

#[test]
fn test_set_3() {
    let file = RiffRam::from_file("test_assets/set_3.riff").unwrap();
    {
        assert_eq!(file.payload_len(), 100);
        assert_eq!(
            ChunkRam::try_from(&file).unwrap().id().as_bytes(),
            riffu::constants::RIFF_ID
        );
        assert_eq!(
            ChunkRam::try_from(&file)
                .unwrap()
                .chunk_type()
                .unwrap()
                .as_bytes(),
            b"smpl"
        );
        assert_eq!(file.iter().unwrap().fold(0, |acc, _| acc + 1), 2);
    }
    {
        let list_1 = file.iter().unwrap().next().unwrap().unwrap();
        assert_eq!(list_1.id().as_bytes(), riffu::constants::LIST_ID);
        assert_eq!(list_1.chunk_type().unwrap().as_bytes(), b"tst1");
        assert_eq!(list_1.iter().fold(0, |acc, _| acc + 1), 2);
        {
            let test = list_1.iter().next().unwrap().unwrap();
            assert_eq!(test.id().as_bytes(), b"test");
            assert_eq!(
                test.get_raw_child().unwrap(),
                "hey this is a test".as_bytes()
            );
        }
        {
            let test = list_1.iter().skip(1).next().unwrap().unwrap();
            assert_eq!(test.id().as_bytes(), b"test");
            assert_eq!(
                test.get_raw_child().unwrap(),
                "hey this is another test".as_bytes()
            );
        }
    }
    {
        let list_1 = file.iter().unwrap().skip(1).next().unwrap().unwrap();
        assert_eq!(list_1.id().as_bytes(), b"seqt");
        assert_eq!(list_1.iter().fold(0, |acc, _| acc + 1), 1);
        assert_eq!(
            list_1.iter().next().unwrap().unwrap().id().as_bytes(),
            b"test"
        );
        assert_eq!(
            list_1
                .iter()
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
    let file = RiffRam::from_file("test_assets/set_4.riff").unwrap();
    {
        assert_eq!(file.payload_len(), 102);
        assert_eq!(
            ChunkRam::try_from(&file).unwrap().id().as_bytes(),
            riffu::constants::RIFF_ID
        );
        assert_eq!(
            ChunkRam::try_from(&file)
                .unwrap()
                .chunk_type()
                .unwrap()
                .as_bytes(),
            b"smpl"
        );
        assert_eq!(file.iter().unwrap().fold(0, |acc, _| acc + 1), 2);
    }
    {
        let list_1 = file.iter().unwrap().next().unwrap().unwrap();
        assert_eq!(list_1.id().as_bytes(), riffu::constants::LIST_ID);
        assert_eq!(list_1.chunk_type().unwrap().as_bytes(), b"tst1");
        assert_eq!(list_1.iter().fold(0, |acc, _| acc + 1), 2);
        {
            let test = list_1.iter().next().unwrap().unwrap();
            assert_eq!(test.id().as_bytes(), b"test");
            assert_eq!(test.get_raw_child().unwrap(), b"hey this is a test");
        }
        {
            let test = list_1.iter().skip(1).next().unwrap().unwrap();
            assert_eq!(test.id().as_bytes(), b"test");
            assert_eq!(test.get_raw_child().unwrap(), b"hey this is another test!");
        }
    }
    {
        let list_1 = file.iter().unwrap().skip(1).next().unwrap().unwrap();
        assert_eq!(list_1.id().as_bytes(), b"seqt");
        assert_eq!(list_1.iter().fold(0, |acc, _| acc + 1), 1);
        assert_eq!(
            list_1.iter().next().unwrap().unwrap().id().as_bytes(),
            b"test"
        );
        assert_eq!(
            list_1
                .iter()
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
fn test_chimes_wav() {
    let file = RiffRam::from_file("test_assets/Chimes.wav").unwrap();
    assert_eq!(b"RIFF", file.id().as_bytes());
    assert_eq!(15924, file.payload_len());
    let expected = vec![(b"fmt ", 16), (b"fact", 4), (b"data", 15876)];
    for (chunk, (expected_name, expected_payload)) in file.iter().unwrap().zip(expected.iter()) {
        let chunk = chunk.unwrap();
        assert_eq!(*expected_name, chunk.id().as_bytes());
        assert_eq!(*expected_payload, chunk.payload_len());
    }
}

#[test]
fn test_canimate_avi() {
    let file = RiffRam::from_file("test_assets/Canimate.avi").unwrap();
    assert_eq!(b"RIFF", file.id().as_bytes());
    assert_eq!(91952, file.payload_len());
    let expected = vec![
        (b"LIST", 1216),
        (b"JUNK", 2840),
        (b"LIST", 87620),
        (b"idx1", 240),
    ];
    for (chunk, (expected_name, expected_payload)) in file.iter().unwrap().zip(expected.iter()) {
        let chunk = chunk.unwrap();
        assert_eq!(*expected_name, chunk.id().as_bytes());
        assert_eq!(*expected_payload, chunk.payload_len());
    }
}
