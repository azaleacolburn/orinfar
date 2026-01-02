use std::iter::once;

use crate::{
    DEBUG,
    buffer::Buffer,
    log,
    undo::{Action, UndoTree},
};

impl Buffer {
    pub fn delete_curr_char(&mut self) {
        if self.cursor == self.rope.len_chars() {
            return;
        }
        // panic!("buffer: {:?}", self.rope.bytes().collect::<Vec<u8>>());
        if self.get_curr_char() == '\n' {
            self.update_list_remove_current();
        }
        self.rope.remove(self.cursor..=self.cursor);
    }

    pub fn replace_curr_char(&mut self, c: char) {
        self.rope.remove(self.cursor..=self.cursor);
        self.rope.insert(self.cursor, &c.to_string());

        self.update_list_use_current_line();
    }

    // Inserts a character at the current position
    pub fn insert_char(&mut self, c: char) {
        if c == '\n' {
            self.update_list_add_current();
        }
        self.rope.insert_char(self.cursor, c);
    }

    // Inserts a newline at the current position, then adds spaces to the new line until the last
    // non-whitespace column lines up
    //
    // Increments cursor accordingly
    //
    // # Returns
    // The contents of the newline including the newline character but not including the contents
    // moved from the previous line
    //
    // It returns the contents inserted into the buffer
    pub fn insert_newline(&mut self) -> String {
        let first_col = self.get_first_non_whitespace_col();
        log!("first_col {}", first_col);
        self.update_list_add_current();
        self.rope.insert_char(self.cursor, '\n');
        self.cursor += 1;
        self.insert_char_n_times(' ', first_col as u8);
        self.cursor += first_col;

        once('\n')
            .chain((0..first_col).map(|_| ' '))
            .collect::<String>()
    }

    pub fn insert_char_n_times(&mut self, c: char, n: u8) {
        if c == '\n' {
            (0..n).for_each(|_| self.update_list_add_current());
        }
        (0..n).for_each(|_| self.insert_char(c));
    }

    pub fn is_last_char(&self) -> bool {
        self.cursor + 1 == self.rope.len_chars()
    }

    pub fn get_curr_char(&self) -> char {
        self.rope.char(self.cursor)
    }

    pub fn get_next_char(&self) -> Option<char> {
        if self.cursor + 1 == self.rope.len_chars() {
            None
        } else {
            Some(self.rope.char(self.cursor + 1))
        }
    }

    pub fn next_and_char(&mut self) -> Option<char> {
        if self.cursor <= self.rope.len_chars() {
            self.cursor += 1;
            return Some(self.rope.char(self.cursor));
        };

        None
    }

    pub fn next_char(&mut self) {
        if self.cursor < self.rope.len_chars() {
            self.cursor += 1;
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

    /// Returns the current zero-indexed column the cursor is on
    pub fn get_col(&self) -> usize {
        log!("get_col cursor: {}, {:?}", self.cursor, self.rope);
        let start_idx = self.get_start_of_line();
        self.cursor - start_idx
    }

    // This is where we are
    pub fn set_col(&mut self, col: usize) {
        log!("\nset_col cursor: {}", self.cursor);
        let start_idx = self.get_start_of_line();
        self.cursor = start_idx + col;
        log!(
            "start_of_line: {}\ncol: {}\nnew_cursor: {} len: {}\n",
            start_idx,
            col,
            self.cursor,
            self.rope.len_chars()
        );
    }

    pub fn get_row(&self) -> usize {
        self.rope.char_to_line(self.cursor)
    }

    pub fn set_row(&mut self, row: usize) {
        let curr_row = self.get_row();
        if curr_row == row || self.rope.len_lines() <= row {
            return;
        }

        let col = self.get_col();
        let end_next_row = self.get_end_of_n_line(row);
        let start_of_next_row = self.rope.line_to_char(row);

        let new_position = usize::min(start_of_next_row + col, end_next_row);
        self.cursor = new_position;
        // Subtracting a signed integer variable from a usize is annoying
        // if curr_row < row {
        //     while curr_row != row && self.cursor + 1 < self.rope.len_chars() {
        //         if self.rope.char(self.cursor) == '\n' {
        //             curr_row += 1;
        //         }
        //         self.cursor += 1;
        //     }
        // } else {
        //     while curr_row != row && self.cursor - 1 < self.rope.len_chars() {
        //         if self.rope.char(self.cursor) == '\n' {
        //             curr_row -= 1;
        //         }
        //         self.cursor -= 1;
        //     }
        // };
    }

    pub fn count_spaces_backwards(&self) -> usize {
        let mut space_count = 0;
        let mut idx = self.cursor;
        while let Some(c) = self.rope.get_char(idx)
            && c == ' '
            && idx > 0
        {
            idx -= 1;
            space_count += 1;
        }

        space_count
    }

    // Returns the deleted string
    pub fn delete_to_4_spaces_alignment(&mut self, space_count: usize) -> String {
        let mut deleted = String::with_capacity(4);
        let mut leftover = space_count % 4;
        if leftover == 0 {
            leftover = 4
        }
        assert!(space_count >= leftover);

        // We need to subtract one because we've already decremented the cursor
        self.cursor -= usize::max(leftover, 1) - 1;
        (0..leftover).for_each(|_| {
            deleted.push(self.get_curr_char());
            self.delete_curr_char();
        });

        deleted
    }

    /// Finds all the occurences of a certain substring `text` in the buffer.
    ///
    /// # Returns
    /// A list of the last index of each occurence.
    ///
    /// For example for the string 'hello world' and the substring 'world',
    /// the index `10` will be put in the list.
    pub fn find_occurences(&self, text: &[char]) -> Vec<usize> {
        let mut curr: Vec<char> = Vec::with_capacity(text.len() - 1);
        let mut idxs_of_substitution: Vec<usize> = Vec::with_capacity(4);

        for (i, char) in self.rope.chars().enumerate() {
            if char == text[curr.len()] {
                curr.push(char);
            }
            if curr.len() == text.len() {
                idxs_of_substitution.push(i + 1);
                curr.clear();
            }
        }

        idxs_of_substitution
    }

    // Replaces all instances of the `original` text with the `new` text in the buffer, given a
    // list of indexes at which they occur
    //
    // # Params
    // - `new`: The text to replace `original`
    // - `original`: The text to be replaced.
    // - `idxs_of_substitution`: The index of the end of each of the occurences of `original` (the
    // last character of each).
    // - `undo_tree`: The undo tree to write an action to
    // - `undoing`: Whether or not this replacement is undoing a previous action or whether it's a
    // new action.
    pub fn replace_text(
        &mut self,
        new: String,
        original: String,
        idxs_of_substitution: &[usize],
        undo_tree: &mut UndoTree,
        undoing: bool,
    ) {
        let offset = new.len() as i32 - original.len() as i32;
        for (i, end_idx) in idxs_of_substitution.iter().enumerate() {
            let offset = i as i32 * offset;
            assert!(
                *end_idx as i32 >= original.len() as i32 + offset,
                "end_idx {} original len {} offset {}",
                end_idx,
                original.len(),
                offset
            );
            let start_idx = (*end_idx as i32 - original.len() as i32 + offset) as usize;
            log!("start_idx: {} offset {}", start_idx, offset);

            self.rope
                .remove(start_idx..(*end_idx as i32 + offset) as usize);
            self.rope.insert(start_idx, &new);

            if !undoing {
                let action = Action::replace(&original, &new);
                undo_tree.new_action_merge(action);
            }
        }

        if self.cursor > self.rope.len_chars() {
            self.cursor = self.rope.len_chars();
        }
    }
}
