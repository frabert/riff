extern crate riffu;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use riffu::{
    constants::{LIST_ID, RIFF_ID},
    lazy::riff::ChunkDisk,
};

fn parse_file(_: ()) {
    {
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
    {
        let mut chunk_root = ChunkDisk::from_path("test_assets/set_3.riff").unwrap();
        {
            assert_eq!(chunk_root.payload_len().unwrap(), 100);
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
    {
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
    {
        let mut chunk_root = ChunkDisk::from_path("test_assets/set_1.riff").unwrap();
        assert_eq!(chunk_root.payload_len().unwrap(), 14);
        assert_eq!(chunk_root.id().unwrap().as_bytes(), b"RIFF");
        assert_eq!(chunk_root.chunk_type().unwrap().as_bytes(), b"smpl");
        let expected_content = vec![vec![255]];
        assert_eq!(
            chunk_root.iter().unwrap().fold(0, |acc, _| acc + 1),
            expected_content.len()
        );
        for (chunk, expected) in chunk_root.iter().unwrap().zip(expected_content) {
            let mut chunk = chunk.unwrap();
            assert_eq!(chunk.get_raw_child().unwrap().len(), expected.len());
            assert_eq!(chunk.get_raw_child().unwrap(), expected);
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("RIFF sets => ", |b| b.iter(|| parse_file(black_box(()))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
