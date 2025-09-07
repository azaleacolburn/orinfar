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
            rope: Rope::from(" "),
            cursor: 0,
        }
    }

    pub fn flush(&self) -> Result<()> {
        let mut stdout = stdout();

        let col = self.get_col();
        let row = self.get_row();
        let string = self.to_string();
        let lines = string.lines();

        execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
        lines.for_each(|line| {
            execute!(stdout, Print(line));
            execute!(stdout, MoveDown(1));
            execute!(stdout, MoveToColumn(0));
        });
        execute!(stdout, MoveTo(col as u16, row as u16))?;
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
}

impl Display for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.rope.to_string())
    }
}
