mod tui;
mod compression;

use clap::{Parser, Subcommand};
use compression::encoder::Encoder;
use compression::decoder::Decoder;
use compression::serializer::Serializer;
use std::io::Read;

#[derive(Parser)]
#[command(name = "lz77")]
#[command(about = "LZ77 file compressor", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Сжать файл
    Encode {
        /// Входной файл
        input: String,
        /// Выходной файл
        output: String,
    },
    /// Распаковать файл
    Decode {
        /// Входной файл
        input: String,
        /// Выходной файл
        output: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => {
            tui::run_tui().unwrap();
        }

        Some(Commands::Encode { input, output }) => {
            const CHUNK_SIZE: usize = 4 * 1024 * 1024;

            let original_size = std::fs::metadata(&input)
                .expect("Не удалось прочитать файл").len();

            let enc = Encoder::new(8192, 64);
            let ser = Serializer::new();

            let in_file = std::fs::File::open(&input)
                .expect("Не удалось открыть файл");
            let mut reader = std::io::BufReader::new(in_file);

            let out_file = std::fs::File::create(&output)
                .expect("Не удалось создать файл");
            let mut writer = std::io::BufWriter::new(out_file);

            ser.write_header(&mut writer).unwrap();

            let mut buf = vec![0u8; CHUNK_SIZE];
            loop {
                let n = reader.read(&mut buf).unwrap();
                if n == 0 { break; }
                let tokens = enc.encode(&buf[..n]);
                for token in &tokens {
                    ser.write_token(&mut writer, token).unwrap();
                }
            }

            ser.write_end(&mut writer).unwrap();
            println!("Сжато: {} байт", original_size);
        }

        Some(Commands::Decode { input, output }) => {
            let in_file = std::fs::File::open(&input)
                .expect("Не удалось открыть файл");
            let mut reader = std::io::BufReader::new(in_file);
            let ser = Serializer::new();
            let tokens = ser.read_tokens(&mut reader)
                .expect("Не удалось прочитать токены");

            let dec = Decoder::new();
            let data = dec.decode(&tokens);

            std::fs::write(&output, &data)
                .expect("Не удалось записать файл");

            println!("Распаковано: {} байт", data.len());
        }
    }
}