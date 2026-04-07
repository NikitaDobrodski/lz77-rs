use std::io::{self, BufReader, BufWriter, Read};
use std::fs::File;
use super::app::Mode;
use super::super::compression::encoder::Encoder;
use super::super::compression::decoder::Decoder;
use super::super::compression::serializer::Serializer;

const CHUNK_SIZE: usize = 4 * 1024 * 1024; // 4 МБ

pub fn process(mode: &Mode, input: &str, output: &str) -> io::Result<(usize, usize)> {
    let ser = Serializer::new();

    match mode {
        Mode::Encode => {
            let original_size = std::fs::metadata(input)?.len() as usize;
            let enc = Encoder::new(8192, 64);

            let in_file = File::open(input)?;
            let mut reader = BufReader::new(in_file);

            let out_file = File::create(output)?;
            let mut writer = BufWriter::new(out_file);

            ser.write_header(&mut writer)?;

            let mut buf = vec![0u8; CHUNK_SIZE];
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 { break; }
                let tokens = enc.encode(&buf[..n]);
                for token in &tokens {
                    ser.write_token(&mut writer, token)?;
                }
            }

            ser.write_end(&mut writer)?;
            let result_size = std::fs::metadata(output)?.len() as usize;
            Ok((original_size, result_size))
        }

        Mode::Decode => {
            let original_size = std::fs::metadata(input)?.len() as usize;
            let in_file = File::open(input)?;
            let mut reader = BufReader::new(in_file);
            let tokens = ser.read_tokens(&mut reader)?;
            let dec = Decoder::new();
            let data = dec.decode(&tokens);
            let result_size = data.len();
            std::fs::write(output, &data)?;
            Ok((original_size, result_size))
        }
    }
}