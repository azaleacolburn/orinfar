use anyhow::Result;
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

use crate::Cursor;

// The cursor is always guaranteed to be within the bounds of the buffer
pub struct Buffer {
    pub buff: Vec<Vec<char>>,
    pub cursor: Cursor,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            buff: vec![vec![]],
            cursor: Cursor { col: 0, row: 0 },
        }
    }

    pub fn col(&self) -> usize {
        self.cursor.col
    }

    pub fn row(&self) -> usize {
        self.cursor.row
    }

    pub fn flush(&self) -> Result<()> {
        let mut stdout = stdout();

        execute!(stdout, MoveTo(0, 0), Clear(ClearType::All),)?;
        for row in self.buff.iter() {
            execute!(
                stdout,
                Print(row.clone().into_iter().collect::<String>()),
                MoveDown(1),
                MoveToColumn(0),
            )?;
        }
        execute!(
            stdout,
            MoveTo(self.cursor.col as u16, self.cursor.row as u16)
        )?;
        stdout.flush()?;

        Ok(())
    }

    pub fn clear(&mut self) {
        self.buff = vec![vec![]]
    }

    pub fn is_last_col(&self) -> bool {
        self.buff[self.cursor.row].len() == self.cursor.col + 1
    }
    pub fn is_last_row(&self) -> bool {
        self.buff.len() == self.cursor.row + 1
    }

    pub fn get_curr_char(&self) -> char {
        self.buff[self.cursor.row][self.cursor.col]
    }

    pub fn get_next_char(&self) -> Option<char> {
        if self.is_last_row() || self.is_last_col() {
            return None;
        }
        Some(self.buff[self.cursor.row][self.cursor.col])
    }

    pub fn next_line(&mut self) {
        if self.buff.len() == self.cursor.row + 1 {
            return;
        }
        self.cursor.row += 1;
    }

    pub fn set_col(&mut self, value: usize) {
        self.cursor.col = value;
    }

    pub fn set_row(&mut self, value: usize) {
        self.cursor.row = value;
    }

    pub fn next_col(&mut self) {
        if self.buff[self.cursor.row].len() + 1 == self.cursor.col {
            return;
        }
        self.cursor.col += 1;
    }

    pub fn next_row(&mut self) {
        if self.buff.len() + 1 == self.cursor.row {
            return;
        }
        self.cursor.row += 1;
    }

    pub fn prev_col(&mut self) {
        if self.cursor.col == 0 {
            return;
        }
        self.cursor.col -= 1;
    }

    pub fn prev_row(&mut self) {
        if self.cursor.row == 0 {
            return;
        }
        self.cursor.row -= 1;
    }

    pub fn push_line(&mut self, line: Vec<char>) {
        self.buff.push(line);
    }

    pub fn is_empty_line(&self) -> bool {
        !self.buff[self.cursor.row].is_empty()
    }

    pub fn get_line_end(&mut self) -> Vec<char> {
        match self.is_empty_line() {
            true => self.buff[self.cursor.row].split_off(self.cursor.col),
            false => vec![],
        }
    }

    pub fn insert_line(&mut self, index: usize, line: Vec<char>) {
        self.buff.insert(index, line);
    }

    pub fn remove_line(&mut self, index: usize) -> Vec<char> {
        self.buff.remove(index)
    }

    pub fn remove_char(&mut self, index: usize) -> char {
        self.buff[self.cursor.row].remove(index)
    }

    pub fn len(&self) -> usize {
        self.buff.len()
    }

    pub fn insert_char_at_cursor(&mut self, c: char) {
        self.buff[self.cursor.row].insert(self.cursor.col, c);
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
