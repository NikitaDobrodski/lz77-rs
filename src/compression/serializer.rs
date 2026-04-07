use crate::compression::token::Token;
use std::io::{self, Read, Write};

pub struct Serializer;

// Маркер конца потока токенов
const END_MARKER: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

impl Serializer {
    pub fn new() -> Self {
        Serializer
    }

    /// Старый метод — для обратной совместимости с тестами
    #[allow(dead_code)]
    pub fn write_tokens<W: Write>(&self, tokens: &[Token], writer: &mut W) -> io::Result<()> {
        self.write_header(writer)?;
        for token in tokens {
            self.write_token(writer, token)?;
        }
        self.write_end(writer)?;
        Ok(())
    }

    /// Записывает заголовок файла
    pub fn write_header<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(b"LZ7S")?;
        Ok(())
    }

    /// Записывает один токен сразу на диск
    pub fn write_token<W: Write>(&self, writer: &mut W, token: &Token) -> io::Result<()> {
        writer.write_all(&token.offset.to_le_bytes())?;
        writer.write_all(&token.length.to_le_bytes())?;
        writer.write_all(&[token.next_char])?;
        Ok(())
    }

    /// Записывает маркер конца потока
    pub fn write_end<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&END_MARKER)?;
        Ok(())
    }

    /// Читает токены потоково
    pub fn read_tokens<R: Read>(&self, reader: &mut R) -> io::Result<Vec<Token>> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        match &magic {
            b"LZ77" => self.read_tokens_legacy(reader),
            b"LZ7S" => self.read_tokens_streaming(reader),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Not a valid LZ77 file",
            )),
        }
    }

    fn read_tokens_legacy<R: Read>(&self, reader: &mut R) -> io::Result<Vec<Token>> {
        let mut count_bytes = [0u8; 4];
        reader.read_exact(&mut count_bytes)?;
        let count = u32::from_le_bytes(count_bytes) as usize;

        let mut tokens = Vec::with_capacity(count);
        for _ in 0..count {
            let mut buf = [0u8; 5];
            reader.read_exact(&mut buf)?;
            let offset = u16::from_le_bytes([buf[0], buf[1]]);
            let length = u16::from_le_bytes([buf[2], buf[3]]);
            tokens.push(Token::new(offset, length, buf[4]));
        }
        Ok(tokens)
    }

    fn read_tokens_streaming<R: Read>(&self, reader: &mut R) -> io::Result<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            let mut buf = [0u8; 5];
            match reader.read_exact(&mut buf) {
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }

            if buf[0] == 0xFF && buf[1] == 0xFF && buf[2] == 0xFF && buf[3] == 0xFF {
                break;
            }

            let offset = u16::from_le_bytes([buf[0], buf[1]]);
            let length = u16::from_le_bytes([buf[2], buf[3]]);
            tokens.push(Token::new(offset, length, buf[4]));
        }
        Ok(tokens)
    }
}