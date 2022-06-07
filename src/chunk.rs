use crc::{Crc, CRC_32_ISO_HDLC};
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::Read;
use thiserror::Error;

use crate::chunk_type::{ChunkType, ChunkTypeError};

#[derive(Debug, Error)]
pub enum ChunkError {
    #[error("IO Error converting from bytes: {0}")]
    InvalidChunkData(#[from] io::Error),
    #[error("Non UTf-8 characters found: {0}")]
    NonUTf8Characters(String),
    #[error("Bad ChunkType: {0}")]
    BadChunkType(#[from] ChunkTypeError),
    #[error("Checksum error")]
    ChecksumError,
}

#[derive(Clone, Debug)]
pub struct Chunk {
    chunk_type: ChunkType,
    data: Vec<u8>,
    length: u32,
    crc: u32,
}

impl TryFrom<&[u8]> for Chunk {
    type Error = ChunkError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let input_stream = &mut &*value;
        let mut buf: [u8; 4] = [0, 0, 0, 0];
        input_stream.read_exact(&mut buf)?;
        let length = u32::from_be_bytes(buf);
        input_stream.read_exact(&mut buf)?;
        let chunk_type = ChunkType::try_from(buf)?;
        let mut data = vec![0u8; length as usize];
        input_stream.read_exact(&mut data)?;
        input_stream.read_exact(&mut buf)?;
        let crc = u32::from_be_bytes(buf);
        if crc != Self::CRC.checksum(&[&chunk_type.bytes()[..], &data.clone()].concat()) {
            return Err(ChunkError::ChecksumError);
        }
        Ok(Self {
            chunk_type,
            data,
            length,
            crc,
        })
    }
}

impl TryFrom<Chunk> for String {
    type Error = ChunkError;

    fn try_from(value: Chunk) -> Result<Self, Self::Error> {
        match std::str::from_utf8(value.data()) {
            Ok(s) => Ok(s.to_string()),
            Err(e) => Err(ChunkError::NonUTf8Characters(e.to_string())),
        }
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Chunk {
    pub const CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

    pub fn new(chunk_type: ChunkType, data: &[u8]) -> Self {
        let crc = Self::CRC.checksum(&[&chunk_type.bytes()[..], data].concat());
        Self {
            chunk_type,
            data: data.to_vec(),
            length: data.len() as u32,
            crc,
        }
    }
    pub fn length(&self) -> u32 {
        self.length
    }
    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    pub fn crc(&self) -> u32 {
        self.crc
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        self.length
            .to_be_bytes()
            .iter()
            .chain(self.chunk_type.bytes().iter())
            .chain(self.data.iter())
            .chain(self.crc.to_be_bytes().iter())
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk_data() -> Vec<u8> {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!".as_bytes();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = Chunk::try_from(testing_chunk_data().as_ref()).unwrap();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = Chunk::try_from(testing_chunk_data().as_ref()).unwrap();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = Chunk::try_from(testing_chunk_data().as_ref()).unwrap();
        let chunk_string: String = chunk.try_into().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = Chunk::try_from(testing_chunk_data().as_ref()).unwrap();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let chunk = Chunk::try_from(testing_chunk_data().as_ref()).unwrap();
        let chunk_string: String = chunk.clone().try_into().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let chunk: Chunk = TryFrom::try_from(testing_chunk_data().as_ref()).unwrap();
        let _chunk_string = format!("{}", chunk);
    }
}
