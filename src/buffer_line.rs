use ropey::{Rope, RopeSlice};

use crate::buffer::Buffer;

impl Buffer {
    pub fn is_empty_line(&self) -> bool {
        self.get_end_of_line() == self.get_start_of_line() + 1
    }

    pub fn remove_n_line(&mut self, n: usize) {
        let start_index = self.get_start_of_char_line(n);
        let end_index = self.get_end_of_n_line(n);
        self.rope.remove(start_index..=end_index)
    }

    /// Removes the line that the given character is on
    pub fn remove_char_line(&mut self, n: usize) {
        let start_index = self.get_start_of_char_line(n);
        let end_index = self.get_end_of_n_line(n);
        self.rope.remove(start_index..=end_index)
    }

    pub fn remove_curr_line(&mut self) {
        self.remove_char_line(self.cursor)
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

    pub fn get_start_of_n_line(&self, n: usize) -> usize {
        self.rope.line_to_char(self.rope.char_to_line(n))
    }

    pub fn get_start_of_char_line(&self, n: usize) -> usize {
        self.rope.line_to_char(self.rope.char_to_line(n))
    }

    pub fn get_start_of_line(&self) -> usize {
        self.get_start_of_char_line(self.cursor)
    }

    pub fn start_of_line(&mut self) {
        self.set_col(self.get_start_of_char_line(self.cursor))
    }

    pub fn get_end_of_n_line(&self, n: usize) -> usize {
        self.rope.line_to_char(self.rope.char_to_line(n) + 1) - 1
    }

    pub fn get_end_of_line(&self) -> usize {
        self.get_end_of_n_line(self.cursor)
    }

    pub fn end_of_line(&mut self) {
        self.set_col(self.get_end_of_line());
    }

    pub fn find_char_in_current_line(&self, c: char) -> Option<usize> {
        let line = self.get_curr_line();
        line.chars().position(|ch| ch == c)
    }

    pub fn push_line(&mut self, line: &str) {
        self.rope.append(Rope::from(line));
    }

    pub fn push_slice(&mut self, rope: RopeSlice<'_>) {
        self.rope.append(rope.into());
    }

    pub fn get_curr_line(&self) -> RopeSlice<'_> {
        self.rope.line(self.rope.char_to_line(self.cursor))
    }

    pub fn prev_line(&mut self) {
        self.set_row(self.get_row() - 1);
    }
}
