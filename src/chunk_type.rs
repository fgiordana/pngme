use std::convert::TryFrom;
use std::fmt::Formatter;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChunkTypeError {
    #[error("Characters can only be alphabetic (A-Z, a-z)")]
    NonAlphabeticCharacters,
    #[error("String should be exactly 4 characters, found: {0}")]
    InvalidStringLength(usize),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChunkType {
    code: [u8; 4],
}

impl ChunkType {
    pub fn bytes(&self) -> [u8; 4] {
        self.code
    }
    pub fn is_critical(&self) -> bool {
        self.code[0].is_ascii_uppercase()
    }
    pub fn is_public(&self) -> bool {
        self.code[1].is_ascii_uppercase()
    }
    pub fn is_reserved_bit_valid(&self) -> bool {
        self.code[2].is_ascii_uppercase()
    }
    pub fn is_safe_to_copy(&self) -> bool {
        self.code[3].is_ascii_lowercase()
    }
    pub fn is_valid(&self) -> bool {
        self.is_reserved_bit_valid()
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = ChunkTypeError;

    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        if value.into_iter().any(|x| !x.is_ascii_alphabetic()) {
            return Err(ChunkTypeError::NonAlphabeticCharacters);
        }
        Ok(Self { code: value })
    }
}

impl FromStr for ChunkType {
    type Err = ChunkTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.chars().into_iter().any(|x| !x.is_ascii_alphabetic()) {
            return Err(ChunkTypeError::NonAlphabeticCharacters);
        }
        if s.len() != 4 {
            return Err(ChunkTypeError::InvalidStringLength(s.len()));
        }
        Ok(Self {
            code: s.as_bytes().try_into().unwrap(),
        })
    }
}

impl std::fmt::Display for ChunkType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", std::str::from_utf8(&self.code).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    fn test_chunk_type_from_bytes() {
        let expected: [u8; 4] = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        assert_eq!(expected, actual.bytes());
    }

    #[test]
    fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_chunk_type_is_critical() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk_type.is_critical());
    }

    #[test]
    fn test_chunk_type_is_not_critical() {
        let chunk_type = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk_type.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk_type = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk_type.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk_type.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk_type.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk_type = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk_type.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk_type.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk_type = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk_type.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk_type.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk_type = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk_type.is_valid());

        let chunk_type = ChunkType::from_str("Ru1t");
        assert!(chunk_type.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk_type.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
