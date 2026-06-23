use crate::{
    buffer::Buffer,
    undo::{Action, UndoTree},
};

use ropey::{Rope, RopeSlice};
use std::iter::once;

impl Buffer {
    pub fn is_empty_line(&self) -> bool {
        self.get_end_of_line() == self.get_start_of_line()
    }

    /// Returns the number of lines in the buffer
    pub fn len(&self) -> usize {
        self.rope.len_lines()
    }

    /// Returns the first index (absolute) of the line represented by the given `line_idx`
    // NOTE
    // We need to allow this because it's used only by another unused function
    // In the future we may need to open an issue on `cargo-clippy` about this
    #[allow(dead_code)]
    pub fn get_start_of_n_line(&self, line_idx: usize) -> usize {
        self.rope.line_to_char(line_idx)
    }

    /// Returns the first index (absolute) of the line where the given `char_idx` is located
    pub fn get_start_of_char_line(&self, char_idx: usize) -> usize {
        let line_idx = self.rope.char_to_line(char_idx);
        self.rope.line_to_char(line_idx)
    }

    pub fn get_first_non_whitespace_col(&self) -> Option<usize> {
        let mut start_of_line = self.get_start_of_line();
        let end_of_line = self.get_end_of_line();
        let anchor = start_of_line;

        while let Some(c) = self.rope.get_char(start_of_line)
            && c.is_whitespace()
        {
            start_of_line += 1;

            if start_of_line >= end_of_line {
                return None;
            }
        }

        Some(start_of_line - anchor)
    }

    pub fn get_start_of_line(&self) -> usize {
        self.get_start_of_char_line(self.cursor)
    }

    pub fn start_of_line(&mut self) {
        self.cursor = self.get_start_of_char_line(self.cursor);
    }

    /// Returns the absolute index of the end of the given `line`
    /// NOT the last column in the line
    pub fn get_end_of_char_line(&self, mut char: usize) -> usize {
        let len = self.rope.len_chars();

        while char + 1 < len {
            let c = self.rope.char(char);
            if c == '\n' {
                break;
            }
            char += 1;
        }
        char
    }

    pub fn get_end_of_n_line(&self, line: usize) -> usize {
        let idx = self.rope.line_to_char(line);

        self.get_end_of_char_line(idx)
    }

    /// Returns the absolute index of the end of the current line
    /// NOT the last column in the line
    pub fn get_end_of_line(&self) -> usize {
        assert!(self.cursor <= self.rope.len_chars());

        self.get_end_of_char_line(self.cursor)
    }

    /// Called by the '$' motion
    pub fn end_of_line(&mut self) {
        self.cursor = self.get_end_of_line();
    }

    pub fn get_curr_line(&self) -> RopeSlice<'_> {
        assert!(self.cursor <= self.rope.len_chars());

        self.rope.line(self.rope.char_to_line(self.cursor))
    }

    pub fn prev_row(&mut self) {
        if self.get_row() == 0 {
            return;
        }

        self.set_row(self.get_row() - 1);

        let len = self.get_curr_line().len_chars();

        // TODO
        // Update to keep track of maximum col since cursor has moved left or right
        let col = if len > 0 {
            usize::min(self.get_col(), len - 1)
        } else {
            0
        };
        self.set_col(col);
    }

    pub fn next_row(&mut self) {
        if self.is_last_row() {
            return;
        }

        self.set_row(self.get_row() + 1);
    }

    pub fn replace_contents(&mut self, contents: &str, undo_tree: &mut UndoTree) {
        self.has_changed = true;
        self.lines_for_updating.clear();

        // NOTE
        // Make sure to correctly add the trailing newline
        if !contents.is_empty() && contents.chars().nth(contents.len() - 1).unwrap_or('\0') == '\n'
        {
            contents
                .lines()
                .chain(once(""))
                .for_each(|_| self.update_list_add(0));
        } else {
            contents.lines().for_each(|_| self.update_list_add(0));
        }

        if contents.is_empty() {
            self.update_list_add(0);
        }

        let action = Action::insert(0, &contents);
        undo_tree.new_action(action);

        self.rope = Rope::from(contents);
        self.clamp_cursor();
    }
}
