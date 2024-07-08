use std::io::{Read, Write};

pub trait Encodable {
    fn encode<W: Write>(&self, stream: &mut W) -> Result<(), std::io::Error>;

    /// Encode into a vector of bytes.
    fn encode_to_vec(&self) -> Result<Vec<u8>, std::io::Error> {
        let mut bytes = Vec::<u8>::default();
        self.encode(&mut bytes)?;
        Ok(bytes)
    }
}

pub trait Decodable: Sized {
    fn decode<R: Read>(stream: &mut R) -> Result<Self, std::io::Error>;

    /// Decode from a slice of bytes.
    fn decode_from_slice(mut slice: &[u8]) -> Result<Self, std::io::Error> {
        Self::decode(&mut slice)
    }
}