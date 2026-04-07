use crate::compression::token::Token;

pub struct Decoder;

impl Decoder {
    pub fn new() -> Self {
        Decoder
    }

    pub fn decode(&self, tokens: &[Token]) -> Vec<u8> {
        let mut output: Vec<u8> = Vec::new();

        for token in tokens {
            if token.is_literal() {
                output.push(token.next_char);
            } else {
                let start = output.len() - token.offset as usize;

                for i in 0..token.length as usize {
                    let byte = output[start + i];
                    output.push(byte);
                }

                // пишем next_char только если он реальный
                if token.next_char != 0 || token.length == 0 {
                    output.push(token.next_char);
                }
            }
        }

        output
    }
}