use crate::buffer::Buffer;

impl Buffer {
    pub fn insert_char(&mut self, c: char) {
        self.rope.insert_char(self.cursor, c);
    }

    pub fn insert_char_n_times(&mut self, c: char, n: u8) {
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

    pub fn next_char(&mut self) -> Option<char> {
        if self.cursor + 1 < self.rope.len_chars() {
            self.cursor += 1;
            return Some(self.rope.char(self.cursor));
        };

        None
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

    pub fn get_col(&self) -> usize {
        let start_idx = self.get_start_of_line();
        self.cursor - start_idx
    }

    pub fn set_col(&mut self, col: usize) {
        let start_idx = self.get_start_of_line();
        self.cursor = start_idx + col;
    }

    pub fn get_row(&self) -> usize {
        self.rope.char_to_line(self.cursor)
    }

    pub fn set_row(&mut self, row: usize) {
        let mut curr_row = self.get_row();
        if curr_row == row {
            return;
        }
        // Subtracting a signed integer variable from a usize is annoying
        if curr_row < row {
            while curr_row != row && self.cursor + 1 < self.rope.len_chars() {
                if self.rope.char(self.cursor) == '\n' {
                    curr_row += 1;
                }
                self.cursor += 1;
            }
        } else {
            while curr_row != row && self.cursor - 1 < self.rope.len_chars() {
                if self.rope.char(self.cursor) == '\n' {
                    curr_row -= 1;
                }
                self.cursor -= 1;
            }
        };
    }
}
