use crate::buffer::Buffer;

impl Buffer {
    pub fn is_last_char(&self) -> bool {
        self.cursor + 1 == self.rope.len_chars()
    }

    pub fn get_curr_char(&self) -> char {
        self.rope.char(self.cursor)
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

    pub fn get_col(&self) -> usize {
        let start_idx = self.get_start_of_line();
        self.cursor - start_idx
    }

    pub fn set_col(&mut self, col: usize) {
        let start_idx = self.get_start_of_line();
        self.cursor = start_idx + col;
    }

    pub fn get_row(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn set_row(&mut self, row: usize) {
        todo!()
    }
}
