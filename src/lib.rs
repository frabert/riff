//! # riff
//! 
//! `riff` provides utility methods for reading and writing RIFF-formatted files,
//! such as Microsoft Wave, AVI or DLS files.

extern crate byteorder;

use std::fmt;
use std::io;
use std::io::Read;
use std::io::Write;

use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};

/// A chunk id, also known as FourCC
#[derive(PartialEq, Eq, Clone)]
pub struct ChunkId {
  /// The raw bytes of the id
  pub value: [u8; 4]
}

/// The `RIFF` id
pub static RIFF_ID: ChunkId = ChunkId { value: [0x52, 0x49, 0x46, 0x46] };

/// The `LIST` id
pub static LIST_ID: ChunkId = ChunkId { value: [0x4C, 0x49, 0x53, 0x54] };

/// The `seqt` id
pub static SEQT_ID: ChunkId = ChunkId { value: [0x73, 0x65, 0x71, 0x74] };

impl ChunkId {
  /// Returns the value of the id as a string.
  /// 
  /// # Examples
  /// ```
  /// assert_eq!(riff::RIFF_ID.as_str(), "RIFF");
  /// ```
  /// 
  /// # Panics
  /// This function panics when the value does not represent a valid UTF-8 string.
  pub fn as_str(&self) -> &str {
    std::str::from_utf8(&self.value).unwrap()
  }

  /// Creates a new ChunkId from a string.
  /// 
  /// # Examples
  /// ```
  /// # use std::error::Error;
  /// #
  /// # fn try_main() -> Result<(), Box<Error>> {
  /// let chunk_id = riff::ChunkId::new("RIFF")?;
  /// #   Ok(())
  /// # }
  /// #
  /// # fn main() {
  /// #     try_main().unwrap();
  /// # }
  /// ```
  /// 
  /// # Errors
  /// The function fails when the string's length in bytes is not exactly 4.
  pub fn new(s: &str) -> Result<ChunkId, &str> {
    let bytes = s.as_bytes();
    if bytes.len() != 4 {
      Err("Invalid length")
    } else {
      let mut a: [u8; 4] = Default::default();
      a.copy_from_slice(&bytes[..]);
      Ok(ChunkId { value: a })
    }
  }

  /// Whether the id is a valid list chunk id.
  /// 
  /// The function returns true only if the id is one of "RIFF", "LIST" or "seqt"
  /// 
  /// # Examples
  /// ```
  /// # use std::error::Error;
  /// #
  /// # fn try_main() -> Result<(), Box<Error>> {
  /// assert!(riff::ChunkId::new("RIFF")?.is_list());
  /// assert!(!riff::ChunkId::new("test")?.is_list());
  /// #   Ok(())
  /// # }
  /// #
  /// # fn main() {
  /// #     try_main().unwrap();
  /// # }
  /// ```
  pub fn is_list(&self) -> bool {
    self == &RIFF_ID || self == &LIST_ID || self == &SEQT_ID
  }
}

impl fmt::Display for ChunkId {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(f, "'{}'", self.as_str())
  }
}

impl fmt::Debug for ChunkId {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
  }
}

/// A chunk, also known as a form
#[derive(PartialEq, Eq, Debug)]
pub struct Chunk {
  /// The id of the chunk
  pub id: ChunkId,
  /// The contents of the chunk
  pub content: ChunkContent
}

impl Chunk {
  fn new(id: ChunkId, content: ChunkContent) -> Chunk {
    Chunk {
      id: id,
      content: content
    }
  }

  /// Creates a new RIFF chunk.
  /// 
  /// # Examples
  /// ```
  /// # use std::error::Error;
  /// #
  /// # fn try_main() -> Result<(), Box<Error>> {
  /// let data_id = riff::ChunkId::new("test")?;
  /// let riff_id = riff::ChunkId::new("foo ")?;
  /// let data_chunk = riff::Chunk::new_data(data_id, vec![0x00, 0x00]);
  /// let riff_chunk = riff::Chunk::new_list(riff_id, vec![data_chunk]);
  /// #   Ok(())
  /// # }
  /// #
  /// # fn main() {
  /// #     try_main().unwrap();
  /// # }
  pub fn new_riff(form_id: ChunkId, content: Vec<Chunk>) -> Chunk {
    Chunk::new(RIFF_ID.clone(), ChunkContent::List {
      form_type: form_id,
      subchunks: content
    })
  }

  /// Creates a new LIST chunk.
  /// 
  /// # Examples
  /// ```
  /// # use std::error::Error;
  /// #
  /// # fn try_main() -> Result<(), Box<Error>> {
  /// let data_id = riff::ChunkId::new("test")?;
  /// let list_id = riff::ChunkId::new("foo ")?;
  /// let data_chunk = riff::Chunk::new_data(data_id, vec![0x00, 0x00]);
  /// let list_chunk = riff::Chunk::new_list(list_id, vec![data_chunk]);
  /// #   Ok(())
  /// # }
  /// #
  /// # fn main() {
  /// #     try_main().unwrap();
  /// # }
  pub fn new_list(form_id: ChunkId, content: Vec<Chunk>) -> Chunk {
    Chunk::new(LIST_ID.clone(), ChunkContent::List {
      form_type: form_id,
      subchunks: content
    })
  }

  /// Creates a new seqt chunk.
  /// 
  /// # Examples
  /// ```
  /// # use std::error::Error;
  /// #
  /// # fn try_main() -> Result<(), Box<Error>> {
  /// let data_id = riff::ChunkId::new("test")?;
  /// let seqt_id = riff::ChunkId::new("foo ")?;
  /// let data_chunk = riff::Chunk::new_data(data_id, vec![0x00, 0x00]);
  /// let seqt_chunk = riff::Chunk::new_seqt(seqt_id, vec![data_chunk]);
  /// #   Ok(())
  /// # }
  /// #
  /// # fn main() {
  /// #     try_main().unwrap();
  /// # }
  pub fn new_seqt(form_id: ChunkId, content: Vec<Chunk>) -> Chunk {
    Chunk::new(SEQT_ID.clone(), ChunkContent::List {
      form_type: form_id,
      subchunks: content
    })
  }

  /// Creates a new data chunk.
  /// 
  /// # Examples
  /// ```
  /// # use std::error::Error;
  /// #
  /// # fn try_main() -> Result<(), Box<Error>> {
  /// let id = riff::ChunkId::new("test")?;
  /// let chunk = riff::Chunk::new_data(id, vec![0x00, 0x00]);
  /// #   Ok(())
  /// # }
  /// #
  /// # fn main() {
  /// #     try_main().unwrap();
  /// # }
  pub fn new_data(form_id: ChunkId, content: Vec<u8>) -> Chunk {
    Chunk::new(form_id, ChunkContent::Subchunk(content))
  }
}

/// The contents of a chunk
#[derive(PartialEq, Eq, Debug)]
pub enum ChunkContent {
  /// The contents of a `RIFF`, `LIST`, or `seqt` chunk
  List {
    /// The type of list form
    form_type: ChunkId,
    /// The contained subchunks
    subchunks: Vec<Chunk>
  },

  /// The contents of a terminal chunk
  Subchunk(Vec<u8>)
}

fn read_id(reader: &mut Read) -> io::Result<ChunkId> {
  let mut fourcc : [u8; 4] = [0; 4];
  reader.read_exact(&mut fourcc)?;
  Ok(ChunkId { value: fourcc })
}

fn read_header(reader: &mut Read) -> io::Result<(ChunkId, u32)> {
  let id = read_id(reader)?;
  let length = reader.read_u32::<LittleEndian>().unwrap();
  Ok((id, length))
}

/// Reads a chunk. Returns the read chunk and the number of bytes read.
///
/// # Examples
///
/// ```
/// # use std::error::Error;
/// # use std::fs::File;
/// #
/// # fn try_main() -> Result<(), Box<Error>> {
/// let mut reader = File::open("test_assets/test.riff")?;
/// let (chunk, _len) = riff::read_chunk(&mut reader)?;
/// #   Ok(())
/// # }
/// #
/// # fn main() {
/// #     try_main().unwrap();
/// # }
/// ```
/// 
/// # Errors
/// The function will fail if the stream doesn't contain a valid RIFF chunk
pub fn read_chunk(reader: &mut Read) -> io::Result<(Chunk, u32)> {
  let (id, len) = read_header(reader)?;
  if id.is_list() {
    let chunk_type = read_id(reader)?;
    let mut count: u32 = 4;
    let mut data: Vec<Chunk> = Vec::new();
    while count < len {
      let (chunk, chunk_len) = read_chunk(reader)?;
      data.push(chunk);
      count = count + chunk_len + 8;
    }
    Ok((Chunk::new(id, ChunkContent::List { form_type: chunk_type, subchunks: data }), count))
  } else {
    let actual_len = len + len % 2;
    let mut data: Vec<u8> = vec![0; actual_len as usize];
    reader.read_exact(&mut data)?;
    data.resize(len as usize, 0);
    Ok((Chunk::new_data(id, data), actual_len))
  }
}

fn calc_len(chunks: &Vec<Chunk>) -> u32 {
  chunks.iter().fold(0, |acc, x| match &x.content {
    ChunkContent::Subchunk(v) => {
      let len = v.len() as u32;
      acc + len + len % 2 + 8
    },
    ChunkContent::List { form_type: _, subchunks } => acc + 12 + calc_len(&subchunks)
  })
}

/// Writes a chunk to a stream.
/// 
/// # Examples
/// 
/// ```
/// # use std::error::Error;
/// #
/// # fn try_main() -> Result<(), Box<Error>> {
/// let mut writer: Vec<u8> = Vec::new();
/// let chunk_id = riff::ChunkId::new("test")?;
/// let chunk = riff::Chunk::new_riff(chunk_id.clone(), vec![
///   riff::Chunk::new_data(chunk_id.clone(), vec![])
/// ]);
/// riff::write_chunk(&mut writer, &chunk)?;
/// #   Ok(())
/// # }
/// #
/// # fn main() {
/// #     try_main().unwrap();
/// # }
/// ```
pub fn write_chunk(writer: &mut Write, chunk: &Chunk) -> io::Result<()> {
  writer.write(&chunk.id.value)?;
  match &chunk.content {
    ChunkContent::Subchunk(v) => {
      let len = v.len() as u32;
      writer.write_u32::<LittleEndian>(len)?;
      writer.write(&v)?;
      if len % 2 != 0 {
        writer.write(&[0; 1])?;
      }
    },
    ChunkContent::List { form_type, subchunks } => {
      let len = calc_len(&subchunks);
      writer.write_u32::<LittleEndian>(len + 4)?;
      writer.write(&form_type.value)?;
      for subchunk in subchunks {
        write_chunk(writer, subchunk)?;
      }
    }
  }
  Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunkid_from_str() {
        assert_eq!(ChunkId::new("RIFF").unwrap(), RIFF_ID);
        assert_eq!(ChunkId::new("LIST").unwrap(), LIST_ID);
        assert_eq!(ChunkId::new("seqt").unwrap(), SEQT_ID);

        assert_eq!(ChunkId::new("123 ").unwrap(),
          ChunkId { value: [0x31, 0x32, 0x33, 0x20] });

        assert_eq!(ChunkId::new("123"), Err("Invalid length"));
        assert_eq!(ChunkId::new("12345"), Err("Invalid length"));
    }

    #[test]
    fn chunkid_lists() {
        assert!(ChunkId::new("RIFF").unwrap().is_list());
        assert!(ChunkId::new("LIST").unwrap().is_list());
        assert!(ChunkId::new("seqt").unwrap().is_list());
        assert!(!ChunkId::new("test").unwrap().is_list());
    }

    #[test]
    fn chunkid_to_str() {
      assert_eq!(RIFF_ID.as_str(), "RIFF");
      assert_eq!(LIST_ID.as_str(), "LIST");
      assert_eq!(SEQT_ID.as_str(), "seqt");
      assert_eq!(ChunkId::new("123 ").unwrap().as_str(), "123 ");
    }

    #[test]
    fn chunkid_format() {
      assert_eq!(format!("{}", RIFF_ID), "'RIFF'");
      assert_eq!(format!("{}", LIST_ID), "'LIST'");
      assert_eq!(format!("{}", SEQT_ID), "'seqt'");

      assert_eq!(format!("{:?}", RIFF_ID), "'RIFF'");
      assert_eq!(format!("{:?}", LIST_ID), "'LIST'");
      assert_eq!(format!("{:?}", SEQT_ID), "'seqt'");
    }
}