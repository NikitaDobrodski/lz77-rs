use crate::compression::token::Token;
use std::collections::HashMap;

pub struct Encoder {
    window_size: usize,
    lookahead_size: usize,
}

impl Encoder {
    pub fn new(window_size: usize, lookahead_size: usize) -> Self {
        Encoder { window_size, lookahead_size }
    }

    pub fn encode(&self, input: &[u8]) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut pos = 0;

        // Хеш-таблица: 3-байтовый ключ → список позиций где встречался
        let mut hash_table: HashMap<(u8, u8, u8), Vec<usize>> = HashMap::new();

        while pos < input.len() {
            let (offset, length) = self.find_match(input, pos, &hash_table);

            // Регистрируем текущую позицию в хеш-таблице
            self.update_hash(input, pos, &mut hash_table);

            let next_pos = pos + length;

            if next_pos < input.len() {
                let next_char = input[next_pos];
                tokens.push(Token::new(offset as u16, length as u16, next_char));
                // Регистрируем все позиции внутри совпадения
                for i in 1..=length {
                    self.update_hash(input, pos + i, &mut hash_table);
                }
                pos += length + 1;
            } else {
                tokens.push(Token::new(offset as u16, length as u16, 0));
                pos += length.max(1);
            }
        }

        tokens
    }

    fn find_match(
        &self,
        input: &[u8],
        pos: usize,
        hash_table: &HashMap<(u8, u8, u8), Vec<usize>>,
    ) -> (usize, usize) {
        // Нужно минимум 3 байта для хеш-поиска
        if pos + 3 > input.len() {
            return (0, 0);
        }

        let key = (input[pos], input[pos + 1], input[pos + 2]);

        let candidates = match hash_table.get(&key) {
            Some(c) => c,
            None => return (0, 0),
        };

        let mut best_offset = 0;
        let mut best_length = 0;
        let max_length = self.lookahead_size.min(input.len() - pos);
        let window_start = pos.saturating_sub(self.window_size);

        // Проверяем кандидатов с конца (ближайшие совпадения лучше)
        for &candidate in candidates.iter().rev() {
            if candidate < window_start {
                break;
            }

            let mut length = 0;
            while length < max_length
                && input[candidate + length] == input[pos + length]
            {
                length += 1;
                if candidate + length >= pos {
                    break;
                }
            }

            if length > best_length {
                best_length = length;
                best_offset = pos - candidate;
                if best_length == max_length {
                    break;
                }
            }
        }

        (best_offset, best_length)
    }

    fn update_hash(
        &self,
        input: &[u8],
        pos: usize,
        hash_table: &mut HashMap<(u8, u8, u8), Vec<usize>>,
    ) {
        if pos + 3 > input.len() {
            return;
        }
        let key = (input[pos], input[pos + 1], input[pos + 2]);
        let entry = hash_table.entry(key).or_default();
        entry.push(pos);

        // Чистим устаревшие позиции чтобы не копить память
        let window_start = pos.saturating_sub(self.window_size);
        if let Some(first_valid) = entry.iter().position(|&p| p >= window_start) {
            if first_valid > 0 {
                entry.drain(..first_valid);
            }
        }
    }
}