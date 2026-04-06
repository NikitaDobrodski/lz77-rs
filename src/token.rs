/// Один токен LZ77
/// (offset, length, next_char)
/// offset = 0, length = 0 означает литерал — просто символ next_char
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub offset: u16,
    pub length: u16,
    pub next_char: u8,
}

impl Token {
    pub fn new(offset: u16, length: u16, next_char: u8) -> Self {
        Token { offset, length, next_char }
    }

    /// Литерал — нет совпадения в истории
    #[allow(dead_code)]
    pub fn literal(ch: u8) -> Self {
        Token { offset: 0, length: 0, next_char: ch }
    }

    pub fn is_literal(&self) -> bool {
        self.offset == 0 && self.length == 0
    }
}