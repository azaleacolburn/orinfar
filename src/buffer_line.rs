use crate::{buffer::Buffer, log};
use ropey::{Rope, RopeSlice};

impl Buffer {
    pub fn is_empty_line(&self) -> bool {
        self.get_end_of_line() == self.get_start_of_line() + 1
    }

    /// Removes the line represented by the given `line_idx`
    pub fn remove_n_line(&mut self, line_idx: usize) {
        self.update_list_remove(line_idx);

        let start_index = self.get_start_of_n_line(line_idx);
        let end_index = self.get_end_of_n_line(line_idx);
        self.rope.remove(start_index..=end_index)
    }

    /// Returns the number of lines in the buffer
    pub fn len(&self) -> usize {
        self.rope.len_lines()
    }

    /// Returns the first index (absolute) of the line represented by the given `line_idx`
    pub fn get_start_of_n_line(&self, line_idx: usize) -> usize {
        self.rope.line_to_char(line_idx)
    }

    /// Returns the first index (absolute) of the line where the given `char_idx` is located
    pub fn get_start_of_char_line(&self, char_idx: usize) -> usize {
        self.rope.line_to_char(self.rope.char_to_line(char_idx))
    }

    pub fn get_first_non_whitespace_col(&self) -> usize {
        let mut start_of_line = self.get_start_of_line();
        while let Some(c) = self.rope.get_char(start_of_line)
            && c.is_whitespace()
        {
            start_of_line += 1;
        }

        start_of_line
    }

    pub fn first_non_whitespace_col(&self) -> usize {
        let mut start_of_line = self.get_start_of_line();
        let anchor = start_of_line;
        while let Some(c) = self.rope.get_char(start_of_line)
            && c.is_whitespace()
        {
            start_of_line += 1;
        }

        start_of_line - anchor
    }

    pub fn get_start_of_line(&self) -> usize {
        log(format!(
            "get_start_of_line cursor: {}, {:?}",
            self.cursor, self.rope
        ));
        self.get_start_of_char_line(self.cursor)
    }

    pub fn start_of_line(&mut self) {
        log(format!(
            "start_of_line cursor: {}, {:?}",
            self.cursor, self.rope
        ));
        self.set_col(self.get_start_of_char_line(self.cursor))
    }

    /// Returns the absolute index of the end of the given `line`
    /// NOT the last column in the line
    pub fn get_end_of_n_line(&self, line: usize) -> usize {
        let mut idx = self.rope.line_to_char(line);
        let len = self.rope.len_chars();

        while idx + 1 < len {
            let c = self.rope.char(idx);
            if c == '\n' {
                break;
            }
            idx += 1;
        }
        idx
    }

    /// Returns the absolute index of the end of the current line
    /// NOT the last column in the line
    pub fn get_end_of_line(&self) -> usize {
        if self.cursor > self.rope.len_chars() {
            panic!("here lol");
            // return;
        }
        let line = self.rope.char_to_line(self.cursor);
        log(format!(
            "get_end_of(curr)_line: {}, line: {}",
            self.cursor, line
        ));
        self.get_end_of_n_line(line)
    }

    /// Called by the '$' motion
    pub fn end_of_line(&mut self) {
        let end_of_line = self.get_end_of_line();
        log(format!("end_of_line: {}", end_of_line));
        self.cursor = end_of_line;
    }

    pub fn get_until_end_of_line(&self) -> RopeSlice<'_> {
        if self.cursor > self.rope.len_chars() {
            panic!("here lol2");
            // return;
        }
        let line_idx = self.rope.char_to_line(self.cursor);
        let line = self.rope.get_line(line_idx).unwrap();
        line.slice(self.cursor..)
    }

    pub fn find_char_in_current_line(&self, c: char) -> Option<usize> {
        let line = self.get_curr_line();
        line.chars().position(|ch| ch == c)
    }

    pub fn get_curr_line(&self) -> RopeSlice<'_> {
        if self.cursor > self.rope.len_chars() {
            panic!("here lol3");
            // return;
        }
        self.rope.line(self.rope.char_to_line(self.cursor))
    }

    pub fn prev_line(&mut self) {
        self.set_row(self.get_row() - 1);
    }

    pub fn next_line(&mut self) {
        let line = self.get_row();
        log(format!("next_line current_line: {}", line));
        self.set_row(line + 1);
    }
}
