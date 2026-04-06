use crate::token::Token;
use std::io::{self, Read, Write};

pub struct Serializer;

impl Serializer {
    pub fn new() -> Self {
        Serializer
    }

    pub fn write_tokens<W: Write>(&self, tokens: &[Token], writer: &mut W) -> io::Result<()> {
        writer.write_all(b"LZ77")?;

        let count = tokens.len() as u32;
        writer.write_all(&count.to_le_bytes())?;

        for token in tokens {
            writer.write_all(&token.offset.to_le_bytes())?;
            writer.write_all(&token.length.to_le_bytes())?;
            writer.write_all(&[token.next_char])?;
        }

        Ok(())
    }

    pub fn read_tokens<R: Read>(&self, reader: &mut R) -> io::Result<Vec<Token>> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != b"LZ77" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Not a valid LZ77 file",
            ));
        }

        let mut count_bytes = [0u8; 4];
        reader.read_exact(&mut count_bytes)?;
        let count = u32::from_le_bytes(count_bytes) as usize;

        let mut tokens = Vec::with_capacity(count);
        for _ in 0..count {
            let mut buf = [0u8; 5];
            reader.read_exact(&mut buf)?;

            let offset = u16::from_le_bytes([buf[0], buf[1]]);
            let length = u16::from_le_bytes([buf[2], buf[3]]);
            let next_char = buf[4];

            tokens.push(Token::new(offset, length, next_char));
        }

        Ok(tokens)
    }
}