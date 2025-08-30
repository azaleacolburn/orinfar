use anyhow::Result;
use ropey::Rope;
use std::{
    fmt::Display,
    io::{stdout, Write},
};

use crossterm::{
    cursor::{MoveDown, MoveTo, MoveToColumn},
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
        self.buff[self.cursor.row].len() == self.cursor.col + 1
    }
    pub fn is_last_row(&self) -> bool {
        self.buff.len() == self.cursor.row + 1
    }

    // NOTE
    // There isn't a guarantee of being able to index, since you could be on an empty line
    // This is problematic and I don't exactly have a good way of fixing it at the moment
    // Although I do have a few ideas including offsetting each line by one index and just having
    // index 0 be a space or NULL smth
    // I don't love this approach
    // A full datastructure ovehaul is probably in order at some point
    //
    // For now we just have to be careful about calling get_curr_char
    // In the future I might move the empty line check to this function
    pub fn get_curr_char(&self) -> char {
        self.buff[self.cursor.row][self.cursor.col]
    }

    pub fn get_curr_line(&self) -> &[char] {
        &self.buff[self.cursor.row]
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

    pub fn next_line(&mut self) {
        if self.buff.len() == self.cursor.row + 1 {
            return;
        }
        self.cursor.row += 1;
    }

    pub fn get_end_of_row(&self) -> usize {
        let len = self.get_curr_line().len();
        if len == 0 {
            return 0;
        }
        return len - 1;
    }

    pub fn end_of_line(&mut self) {
        let len = self.get_curr_line().len();
        if len == 0 {
            return;
        }
        self.set_col(len - 1);
    }

    pub fn is_empty_line(&self) -> bool {
        self.get_end_of_line() == self.beginning_of_line() + 1
    }

    pub fn remove_curr_line(&mut self) {
        let start_index = self.beginning_of_line();
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

    pub fn beginning_of_line(&self) -> usize {
        self.rope.line_to_char(self.rope.char_to_line(self.cursor))
    }

    pub fn get_end_of_line(&self) -> usize {
        self.rope
            .line_to_char(self.rope.char_to_line(self.cursor) + 1)
            - 1
    }
}

impl Display for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            &self
                .buff
                .iter()
                .map(|line| {
                    let mut c: String = line.iter().collect();
                    c.push('\n');
                    c
                })
                .collect::<String>(),
        )
    }
}
