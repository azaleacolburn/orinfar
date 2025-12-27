use ropey::Rope;
use std::fmt::Display;

// The cursor is always guaranteed to be within the bounds of the buffer
#[derive(Debug, Clone, PartialEq)]
pub struct Buffer {
    pub has_changed: bool,
    pub lines_for_updating: Vec<bool>,
    pub rope: Rope,
    pub cursor: usize,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            has_changed: true,
            lines_for_updating: vec![true],
            rope: Rope::from(""),
            cursor: 0,
        }
    }

    pub fn clear(&mut self) {
        self.rope.remove(0..);
    }

    pub fn is_last_col(&self) -> bool {
        self.cursor + 1 >= self.rope.len_chars() || self.rope.char(self.cursor) == '\n'
    }

    pub fn is_last_row(&self) -> bool {
        self.rope.char_to_line(self.cursor) + 1 == self.rope.len_lines()
    }

    pub fn is_first_row(&self) -> bool {
        self.rope.char_to_line(self.cursor) == 0
    }
}

impl Display for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.rope.to_string())
    }
}
