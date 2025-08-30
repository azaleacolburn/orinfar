use anyhow::Result;
use ropey::{Rope, RopeSlice};
use std::{
    fmt::Display,
    io::{stdout, Write},
};

use crossterm::{
    cursor::MoveTo,
    execute,
    style::Print,
    terminal::{Clear, ClearType},
};

// The cursor is always guaranteed to be within the bounds of the buffer
#[derive(Debug, Clone, PartialEq)]
pub struct Buffer {
    pub rope: Rope,
    pub cursor: usize,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            rope: Rope::new(),
            cursor: 0,
        }
    }

    pub fn flush(&self) -> Result<()> {
        let mut stdout = stdout();

        execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
        execute!(stdout, Print(self.rope.to_string()),)?;
        // TODO This might acturlly be non-trivial 3:
        // execute!(
        //     stdout,
        //     MoveTo(self.cursor.col as u16, self.cursor.row as u16)
        // )?;
        stdout.flush()?;

        Ok(())
    }

    pub fn clear(&mut self) {
        self.rope.remove(0..);
    }

    pub fn is_last_col(&self) -> bool {
        self.cursor + 1 >= self.rope.len_chars() || self.rope.char(self.cursor + 1) == '\n'
    }
    pub fn is_last_row(&self) -> bool {
        self.rope.char_to_line(self.cursor + 1) == self.rope.len_lines()
    }
    pub fn is_last_char(&self) -> bool {
        self.cursor + 1 == self.rope.len_chars()
    }

    pub fn get_curr_char(&self) -> char {
        self.rope.char(self.cursor)
    }

    pub fn get_curr_line(&self) -> RopeSlice<'_> {
        self.rope.line(self.rope.char_to_line(self.cursor))
    }

    pub fn get_next_char(&self) -> Option<char> {
        if self.cursor == 0 {
            None
        } else {
            Some(self.rope.char(self.cursor + 1))
        }
    }

    pub fn next_char(&mut self) -> Option<char> {
        if self.cursor == 0 {
            None
        } else {
            self.cursor += 1;
            Some(self.rope.char(self.cursor))
        }
    }

    pub fn get_prev_char(&self) -> Option<char> {
        if self.cursor == 0 {
            None
        } else {
            Some(self.rope.char(self.cursor - 1))
        }
    }

    pub fn prev_char(&mut self) -> Option<char> {
        if self.cursor == 0 {
            None
        } else {
            self.cursor -= 1;
            Some(self.rope.char(self.cursor))
        }
    }

    pub fn get_end_of_row(&self) -> usize {
        let len = self.get_curr_line().len_chars();
        if len == 0 {
            return 0;
        }
        return len - 1;
    }

    pub fn is_empty_line(&self) -> bool {
        self.get_end_of_line() == self.get_start_of_line() + 1
    }

    pub fn get_col(&self) -> usize {
        let start_idx = self.get_start_of_line();
        self.cursor - start_idx
    }

    pub fn set_col(&mut self, col: usize) {
        let start_idx = self.get_start_of_line();
        self.cursor = start_idx + col;
    }

    pub fn remove_curr_line(&mut self) {
        let start_index = self.get_start_of_line();
        let end_index = self.get_end_of_line();
        self.rope.remove(start_index..=end_index);
    }

    pub fn len(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn delete_curr_char(&mut self) {
        self.rope.remove(self.cursor..self.cursor);
    }

    pub fn replace_curr_char(&mut self, c: char) {
        self.rope.remove(self.cursor..self.cursor);
        self.rope.insert(self.cursor, &c.to_string());
    }

    pub fn get_start_of_line(&self) -> usize {
        self.rope.line_to_char(self.rope.char_to_line(self.cursor))
    }

    pub fn start_of_line(&mut self) {
        self.set_col(self.rope.line_to_char(self.rope.char_to_line(self.cursor)))
    }

    pub fn get_end_of_line(&self) -> usize {
        self.rope
            .line_to_char(self.rope.char_to_line(self.cursor) + 1)
            - 1
    }

    pub fn end_of_line(&mut self) {
        self.set_col(self.get_end_of_line());
    }

    pub fn find_char_in_current_line(&self) -> usize {
        todo!()
    }
}

impl Display for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.rope.to_string())
    }
}
