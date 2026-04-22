use ropey::Rope;
use std::fmt::Display;

// The cursor is always guaranteed to be within the bounds of the buffer
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffer {
    pub has_changed: bool,
    pub lines_for_updating: Vec<bool>,
    pub rope: Rope,
    pub cursor: usize,
    /// The largest column since moving sideways
    pub intended_column: usize,
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            has_changed: true,
            lines_for_updating: vec![true],
            rope: Rope::from(""),
            intended_column: 0,
            cursor: 0,
        }
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
