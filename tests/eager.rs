extern crate riffu;

use riffu::error::RiffResult;

#[test]
fn test_minimal() -> RiffResult<()> {
    let file =
        riffu::eager::riff::Riff::from_file(std::path::PathBuf::from("test_assets/set_1.riff"))?;
    assert_eq!(file.payload_len(), 14);
    assert_eq!(
        riffu::eager::riff::Chunk::from(&file).id().as_str()?,
        "RIFF"
    );
    assert_eq!(
        riffu::eager::riff::Chunk::from(&file)
            .chunk_type()
            .as_str()?,
        "smpl"
    );
    let expected_content = vec![vec![255]];
    assert_eq!(
        file.iter().fold(0, |acc, _| acc + 1),
        expected_content.len()
    );
    for (chunk, expected) in file.iter().zip(expected_content) {
        assert_eq!(chunk.get_raw_child()?.len(), expected.len());
        assert_eq!(chunk.get_raw_child()?, expected);
    }
    assert_eq!(file.iter().skip(1).next(), None);
    Ok(())
}

#[test]
fn test_minimal_2() -> RiffResult<()> {
    let file =
        riffu::eager::riff::Riff::from_file(std::path::PathBuf::from("test_assets/set_2.riff"))?;
    assert_eq!(file.payload_len(), 24);
    assert_eq!(
        riffu::eager::riff::Chunk::from(&file).id().as_str()?,
        "RIFF"
    );
    assert_eq!(
        riffu::eager::riff::Chunk::from(&file)
            .chunk_type()
            .as_str()?,
        "smpl"
    );
    let expected_content = vec![("tst1", vec![255]), ("tst2", vec![238])];
    assert_eq!(
        file.iter().fold(0, |acc, _| acc + 1),
        expected_content.len()
    );
    for (chunk, (name, data)) in file.iter().zip(expected_content) {
        assert_eq!(chunk.id().as_str()?, name);
        assert_eq!(chunk.get_raw_child()?.len(), data.len());
        assert_eq!(chunk.get_raw_child()?, data);
    }
    assert_eq!(file.iter().skip(2).next(), None);
    Ok(())
}

#[test]
fn test_test() -> RiffResult<()> {
    let file =
        riffu::eager::riff::Riff::from_file(std::path::PathBuf::from("test_assets/set_3.riff"))?;
    {
        assert_eq!(file.payload_len(), 100);
        assert_eq!(
            riffu::eager::riff::Chunk::from(&file).id().as_str()?,
            riffu::constants::RIFF_ID
        );
        assert_eq!(
            riffu::eager::riff::Chunk::from(&file)
                .chunk_type()
                .as_str()?,
            "smpl"
        );
        assert_eq!(file.iter().fold(0, |acc, _| acc + 1), 2);
    }
    {
        let list_1 = file.iter().next()?;
        assert_eq!(list_1.id().as_str()?, riffu::constants::LIST_ID);
        assert_eq!(list_1.chunk_type().as_str()?, "tst1");
        assert_eq!(list_1.iter().fold(0, |acc, _| acc + 1), 2);
        {
            let test = list_1.iter().next()?;
            assert_eq!(test.id().as_str()?, "test");
            assert_eq!(test.get_raw_child()?, "hey this is a test".as_bytes());
        }
        {
            let test = list_1.iter().skip(1).next()?;
            assert_eq!(test.id().as_str()?, "test");
            assert_eq!(test.get_raw_child()?, "hey this is another test".as_bytes());
        }
    }
    {
        let list_1 = file.iter().skip(1).next()?;
        assert_eq!(list_1.id().as_str()?, "seqt");
        assert_eq!(list_1.iter().fold(0, |acc, _| acc + 1), 1);
        assert_eq!(list_1.iter().next()?.id().as_str()?, "test");
        assert_eq!(
            list_1.iter().next()?.get_raw_child()?,
            "final test".as_bytes()
        );
    }
    Ok(())
}

#[test]
fn test_test_2() -> RiffResult<()> {
    let file =
        riffu::eager::riff::Riff::from_file(std::path::PathBuf::from("test_assets/set_4.riff"))?;
    {
        assert_eq!(file.payload_len(), 102);
        assert_eq!(
            riffu::eager::riff::Chunk::from(&file).id().as_str()?,
            riffu::constants::RIFF_ID
        );
        assert_eq!(
            riffu::eager::riff::Chunk::from(&file)
                .chunk_type()
                .as_str()?,
            "smpl"
        );
        assert_eq!(file.iter().fold(0, |acc, _| acc + 1), 2);
    }
    {
        let list_1 = file.iter().next()?;
        assert_eq!(list_1.id().as_str()?, riffu::constants::LIST_ID);
        assert_eq!(list_1.chunk_type().as_str()?, "tst1");
        assert_eq!(list_1.iter().fold(0, |acc, _| acc + 1), 2);
        {
            let test = list_1.iter().next()?;
            assert_eq!(test.id().as_str()?, "test");
            assert_eq!(test.get_raw_child()?, "hey this is a test".as_bytes());
        }
        {
            let test = list_1.iter().skip(1).next()?;
            assert_eq!(test.id().as_str()?, "test");
            assert_eq!(
                test.get_raw_child()?,
                "hey this is another test!".as_bytes()
            );
        }
    }
    {
        let list_1 = file.iter().skip(1).next()?;
        assert_eq!(list_1.id().as_str()?, "seqt");
        assert_eq!(list_1.iter().fold(0, |acc, _| acc + 1), 1);
        assert_eq!(list_1.iter().next()?.id().as_str()?, "test");
        assert_eq!(
            list_1.iter().next()?.get_raw_child()?,
            "final test".as_bytes()
        );
    }
    Ok(())
}

#[test]
fn test_chimes_wav() -> RiffResult<()> {
    let file =
        riffu::eager::riff::Riff::from_file(std::path::PathBuf::from("test_assets/Chimes.wav"))?;
    assert_eq!("RIFF", file.id().as_str()?);
    assert_eq!(15924, file.payload_len());
    let expected = vec![("fmt ", 16), ("fact", 4), ("data", 15876)];
    for (ref chunk, (expected_name, expected_payload)) in file.iter().zip(expected.iter()) {
        assert_eq!(*expected_name, chunk.id().as_str()?);
        assert_eq!(*expected_payload, chunk.payload_len());
    }
    Ok(())
}
