use lz77_rs::compression::encoder::Encoder;
use lz77_rs::compression::decoder::Decoder;
use lz77_rs::compression::serializer::Serializer;
use std::io::Cursor;

fn roundtrip(input: &[u8]) -> bool {
    let enc = Encoder::new(255, 15);
    let dec = Decoder::new();
    let tokens = enc.encode(input);
    let decoded = dec.decode(&tokens);
    input == decoded.as_slice()
}

#[test]
fn test_basic() {
    assert!(roundtrip(b"abracadabra"));
}

#[test]
fn test_empty() {
    assert!(roundtrip(b""));
}

#[test]
fn test_single_char() {
    assert!(roundtrip(b"a"));
}

#[test]
fn test_all_same() {
    assert!(roundtrip(b"aaaaaaaaaa"));
}

#[test]
fn test_no_repetitions() {
    assert!(roundtrip(b"abcdefghij"));
}

#[test]
fn test_binary_data() {
    let input: Vec<u8> = (0u8..=255u8).collect();
    assert!(roundtrip(&input));
}

#[test]
fn test_long_repetition() {
    let input = b"abcabcabcabcabcabcabcabc";
    assert!(roundtrip(input));
}

#[test]
fn test_serialization_roundtrip() {
    let input = b"abracadabra";
    let enc = Encoder::new(255, 15);
    let dec = Decoder::new();
    let ser = Serializer::new();

    let tokens = enc.encode(input);

    let mut buf = Vec::new();
    ser.write_tokens(&tokens, &mut buf).unwrap();

    let mut cursor = Cursor::new(buf);
    let restored_tokens = ser.read_tokens(&mut cursor).unwrap();

    let decoded = dec.decode(&restored_tokens);
    assert_eq!(input.as_ref(), decoded.as_slice());
}