extern crate riff;

use std::convert::TryFrom;

#[test]
fn test_minimal() {
    let file = riff::lazy::riff::Riff::from_path("test_assets/set_1.riff").unwrap();
    let chunk_root = riff::lazy::riff::Chunk::try_from(file).unwrap();
    assert_eq!(chunk_root.payload_len(), 14);
    assert_eq!(chunk_root.id().as_str(), "RIFF");
    assert_eq!(chunk_root.chunk_type().as_ref().unwrap().as_str(), "smpl");
    let expected_content = vec![vec![255]];
    assert_eq!(
        chunk_root.iter().fold(0, |acc, _| acc + 1),
        expected_content.len()
    );
    chunk_root
        .iter()
        .zip(expected_content)
        .for_each(|(chunk, expected)| {
            assert_eq!(chunk.get_raw_child().unwrap().len(), expected.len());
            assert_eq!(chunk.get_raw_child().unwrap(), expected);
        });
}

#[test]
fn test_minimal_2() {
    let file = riff::lazy::riff::Riff::from_path("test_assets/set_2.riff").unwrap();
    let chunk_root = riff::lazy::riff::Chunk::try_from(file).unwrap();
    assert_eq!(chunk_root.payload_len(), 24);
    assert_eq!(chunk_root.id().as_str(), "RIFF");
    assert_eq!(chunk_root.chunk_type().as_ref().unwrap().as_str(), "smpl");
    let expected_content = vec![("tst1", vec![255]), ("tst2", vec![238])];
    assert_eq!(
        chunk_root.iter().fold(0, |acc, _| acc + 1),
        expected_content.len()
    );
    chunk_root
        .iter()
        .zip(expected_content)
        .for_each(|(chunk, (name, data))| {
            assert_eq!(chunk.id().as_str(), name);
            assert_eq!(chunk.get_raw_child().unwrap().len(), data.len());
            assert_eq!(chunk.get_raw_child().unwrap(), data);
        });
}

#[test]
fn test_test() {
    let file = riff::lazy::riff::Riff::from_path("test_assets/set_3.riff").unwrap();
    let chunk_root = riff::lazy::riff::Chunk::try_from(file).unwrap();
    {
        assert_eq!(chunk_root.payload_len(), 100);
        assert_eq!(chunk_root.id().as_str(), riff::constants::RIFF_ID);
        assert_eq!(chunk_root.chunk_type().as_ref().unwrap().as_str(), "smpl");
        assert_eq!(chunk_root.iter().fold(0, |acc, _| acc + 1), 2);
    }
    {
        let list_1 = chunk_root.iter().next().unwrap();
        assert_eq!(list_1.id().as_str(), riff::constants::LIST_ID);
        assert_eq!(list_1.chunk_type().as_ref().unwrap().as_str(), "tst1");
        assert_eq!(list_1.iter().fold(0, |acc, _| acc + 1), 2);
        {
            let test = list_1.iter().next().unwrap();
            assert_eq!(test.id().as_str(), "test");
            assert_eq!(
                test.get_raw_child().unwrap(),
                "hey this is a test".as_bytes()
            );
        }
        {
            let test = list_1.iter().skip(1).next().unwrap();
            assert_eq!(test.id().as_str(), "test");
            assert_eq!(
                test.get_raw_child().unwrap(),
                "hey this is another test".as_bytes()
            );
        }
    }
    {
        let list_1 = chunk_root.iter().skip(1).next().unwrap();
        assert_eq!(list_1.id().as_str(), "seqt");
        assert_eq!(list_1.iter().fold(0, |acc, _| acc + 1), 1);
        assert_eq!(list_1.iter().next().unwrap().id().as_str(), "test");
        assert_eq!(
            list_1.iter().next().unwrap().get_raw_child().unwrap(),
            "final test".as_bytes()
        );
    }
}

#[test]
fn test_test_2() {
    let file = riff::lazy::riff::Riff::from_path("test_assets/set_4.riff").unwrap();
    let chunk_root = riff::lazy::riff::Chunk::try_from(file).unwrap();
    {
        assert_eq!(chunk_root.payload_len(), 102);
        assert_eq!(chunk_root.id().as_str(), riff::constants::RIFF_ID);
        assert_eq!(chunk_root.chunk_type().as_ref().unwrap().as_str(), "smpl");
        assert_eq!(chunk_root.iter().fold(0, |acc, _| acc + 1), 2);
    }
    {
        let list_1 = chunk_root.iter().next().unwrap();
        assert_eq!(list_1.id().as_str(), riff::constants::LIST_ID);
        assert_eq!(list_1.chunk_type().as_ref().unwrap().as_str(), "tst1");
        assert_eq!(list_1.iter().fold(0, |acc, _| acc + 1), 2);
        {
            let test = list_1.iter().next().unwrap();
            assert_eq!(test.id().as_str(), "test");
            assert_eq!(
                test.get_raw_child().unwrap(),
                "hey this is a test".as_bytes()
            );
        }
        {
            let test = list_1.iter().skip(1).next().unwrap();
            assert_eq!(test.id().as_str(), "test");
            assert_eq!(
                test.get_raw_child().unwrap(),
                "hey this is another test!".as_bytes()
            );
        }
    }
    {
        let list_1 = chunk_root.iter().skip(1).next().unwrap();
        assert_eq!(list_1.id().as_str(), "seqt");
        assert_eq!(list_1.iter().fold(0, |acc, _| acc + 1), 1);
        assert_eq!(list_1.iter().next().unwrap().id().as_str(), "test");
        assert_eq!(
            list_1.iter().next().unwrap().get_raw_child().unwrap(),
            "final test".as_bytes()
        );
    }
}
