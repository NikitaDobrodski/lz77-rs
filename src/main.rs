mod token;
mod encoder;
mod decoder;
mod serializer;

use clap::{Parser, Subcommand};
use encoder::Encoder;
use decoder::Decoder;
use serializer::Serializer;
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(Parser)]
#[command(name = "lz77")]
#[command(about = "LZ77 file compressor", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
        Commands::Encode { input, output } => {
            let data = std::fs::read(&input).unwrap_or_else(|e| {
                eprintln!("Error reading '{}': {}", input, e);
                std::process::exit(1);
            });

            let enc = Encoder::new(255, 15);
            let tokens = enc.encode(&data);

            let file = File::create(&output).unwrap_or_else(|e| {
                eprintln!("Error creating '{}': {}", output, e);
                std::process::exit(1);
            });
            let mut writer = BufWriter::new(file);
            let ser = Serializer::new();
            ser.write_tokens(&tokens, &mut writer).unwrap_or_else(|e| {
                eprintln!("Error writing: {}", e);
                std::process::exit(1);
            });

            println!(
                "Encoded: {} bytes → {} tokens",
                data.len(),
                tokens.len()
            );
        }

        Commands::Decode { input, output } => {
            let file = File::open(&input).unwrap_or_else(|e| {
                eprintln!("Error opening '{}': {}", input, e);
                std::process::exit(1);
            });
            let mut reader = BufReader::new(file);
            let ser = Serializer::new();
            let tokens = ser.read_tokens(&mut reader).unwrap_or_else(|e| {
                eprintln!("Error reading tokens: {}", e);
                std::process::exit(1);
            });

            let dec = Decoder::new();
            let data = dec.decode(&tokens);

            std::fs::write(&output, &data).unwrap_or_else(|e| {
                eprintln!("Error writing '{}': {}", output, e);
                std::process::exit(1);
            });

            println!("Decoded: {} tokens → {} bytes", tokens.len(), data.len());
        }
    }
}