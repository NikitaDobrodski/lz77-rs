use crate::token::Token;

pub struct Encoder {
    window_size: usize,    // размер search buffer (история)
    lookahead_size: usize, // размер lookahead buffer
}

impl Encoder {
    pub fn new(window_size: usize, lookahead_size: usize) -> Self {
        Encoder { window_size, lookahead_size }
    }

    pub fn encode(&self, input: &[u8]) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut pos = 0;

        while pos < input.len() {
            let (offset, length) = self.find_longest_match(input, pos);

            let next_pos = pos + length;

            if next_pos < input.len() {
                // есть следующий символ — стандартный токен
                let next_char = input[next_pos];
                tokens.push(Token::new(offset as u16, length as u16, next_char));
                pos += length + 1;
            } else {
                // совпадение дошло до конца — next_char не нужен
                tokens.push(Token::new(offset as u16, length as u16, 0));
                pos += length;
            }
        }

        tokens
    }

    fn find_longest_match(&self, input: &[u8], pos: usize) -> (usize, usize) {
        // граница начала search buffer
        let search_start = pos.saturating_sub(self.window_size);

        let mut best_offset = 0;
        let mut best_length = 0;

        for i in search_start..pos {
            let mut length = 0;
            let max_length = self.lookahead_size.min(input.len() - pos);

            while length < max_length && input[i + length] == input[pos + length] {
                length += 1;

                // защита от выхода за границу search buffer
                if i + length >= pos {
                    break;
                }
            }

            if length > best_length {
                best_length = length;
                best_offset = pos - i;
            }
        }

        (best_offset, best_length)
    }
}