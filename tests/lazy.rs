extern crate riffu;

use riffu::{
    constants::{LIST_ID, RIFF_ID},
    error::RiffResult,
    lazy::riff::{ChunkDisk, RiffDisk},
};
use std::convert::TryFrom;

#[test]
fn test_set_1() -> RiffResult<()> {
    let file = RiffDisk::from_file("test_assets/set_1.riff")?;
    let chunk_root = ChunkDisk::try_from(file)?;
    assert_eq!(chunk_root.payload_len(), 14);
    assert_eq!(chunk_root.id().as_str()?, "RIFF");
    assert_eq!(chunk_root.chunk_type().as_ref()?.as_str()?, "smpl");
    let expected_content = vec![("test", vec![255])];
    assert_eq!(
        chunk_root.iter()?.fold(0, |acc, _| acc + 1),
        expected_content.len()
    );
    for (chunk, (expected_name, expected_data)) in chunk_root.iter()?.zip(expected_content) {
        let chunk = chunk?;
        assert_eq!("test", expected_name);
        assert_eq!(chunk.get_raw_child()?.len(), expected_data.len());
        assert_eq!(chunk.get_raw_child()?, expected_data);
    }
    Ok(())
}

#[test]
fn test_set_2() -> RiffResult<()> {
    let file = RiffDisk::from_file("test_assets/set_2.riff")?;
    let chunk_root = ChunkDisk::try_from(file)?;
    assert_eq!(chunk_root.payload_len(), 24);
    assert_eq!(chunk_root.id().as_str()?, "RIFF");
    assert_eq!(chunk_root.chunk_type().as_ref()?.as_str()?, "smpl");
    let expected_content = vec![("tst1", vec![255]), ("tst2", vec![238])];
    assert_eq!(
        chunk_root.iter()?.fold(0, |acc, _| acc + 1),
        expected_content.len()
    );
    for (chunk, (name, data)) in chunk_root.iter()?.zip(expected_content) {
        let chunk = chunk?;
        assert_eq!(chunk.id().as_str()?, name);
        assert_eq!(chunk.get_raw_child()?.len(), data.len());
        assert_eq!(chunk.get_raw_child()?, data);
    }
    Ok(())
}

#[test]
fn test_set_3() -> RiffResult<()> {
    let file = RiffDisk::from_file("test_assets/set_3.riff")?;
    let chunk_root = ChunkDisk::try_from(file)?;
    {
        assert_eq!(chunk_root.payload_len(), 100);
        assert_eq!(chunk_root.id().as_str()?, riffu::constants::RIFF_ID);
        assert_eq!(chunk_root.chunk_type().as_ref()?.as_str()?, "smpl");
        assert_eq!(chunk_root.iter()?.fold(0, |acc, _| acc + 1), 2);
    }
    {
        let list_1 = chunk_root.iter()?.next()??;
        assert_eq!(list_1.id().as_str()?, riffu::constants::LIST_ID);
        assert_eq!(list_1.chunk_type().as_ref()?.as_str()?, "tst1");
        assert_eq!(list_1.iter()?.fold(0, |acc, _| acc + 1), 2);
        {
            let test = list_1.iter()?.next()??;
            assert_eq!(test.id().as_str()?, "test");
            assert_eq!(test.get_raw_child()?, "hey this is a test".as_bytes());
        }
        {
            let test = list_1.iter()?.skip(1).next()??;
            assert_eq!(test.id().as_str()?, "test");
            assert_eq!(test.get_raw_child()?, "hey this is another test".as_bytes());
        }
    }
    {
        let list_1 = chunk_root.iter()?.skip(1).next()??;
        assert_eq!(list_1.id().as_str()?, "seqt");
        assert_eq!(list_1.iter()?.fold(0, |acc, _| acc + 1), 1);
        assert_eq!(list_1.iter()?.next()??.id().as_str()?, "test");
        assert_eq!(
            list_1.iter()?.next()??.get_raw_child()?,
            "final test".as_bytes()
        );
    }
    Ok(())
}

#[test]
fn test_set_4() -> RiffResult<()> {
    let file = RiffDisk::from_file("test_assets/set_4.riff")?;
    let chunk_root = ChunkDisk::try_from(file)?;
    {
        assert_eq!(chunk_root.payload_len(), 102);
        assert_eq!(chunk_root.id().as_str()?, RIFF_ID);
        assert_eq!(chunk_root.chunk_type().as_ref()?.as_str()?, "smpl");
        assert_eq!(chunk_root.iter()?.fold(0, |acc, _| acc + 1), 2);
    }
    {
        let list_1 = chunk_root.iter()?.next()??;
        assert_eq!(list_1.id().as_str()?, LIST_ID);
        assert_eq!(list_1.chunk_type().as_ref()?.as_str()?, "tst1");
        assert_eq!(list_1.iter()?.fold(0, |acc, _| acc + 1), 2);
        {
            let test = list_1.iter()?.next()??;
            assert_eq!(test.id().as_str()?, "test");
            assert_eq!(test.get_raw_child()?, "hey this is a test".as_bytes());
        }
        {
            let test = list_1.iter()?.skip(1).next()??;
            assert_eq!(test.id().as_str()?, "test");
            assert_eq!(
                test.get_raw_child()?,
                "hey this is another test!".as_bytes()
            );
        }
    }
    {
        let list_1 = chunk_root.iter()?.skip(1).next()??;
        assert_eq!(list_1.id().as_str()?, "seqt");
        assert_eq!(list_1.iter()?.fold(0, |acc, _| acc + 1), 1);
        assert_eq!(list_1.iter()?.next()??.id().as_str()?, "test");
        assert_eq!(
            list_1.iter()?.next()??.get_raw_child()?,
            "final test".as_bytes()
        );
    }
    Ok(())
}
