use anyhow::Result;
use crossterm::{
    cursor::{MoveDown, MoveTo, MoveToColumn, MoveToRow},
    execute,
    style::Print,
    terminal::{Clear, ClearType},
};
use std::{
    io::{stdout, Write},
    path::PathBuf,
};

use crate::{
    buffer::{self, Buffer},
    log,
    status_bar::StatusBar,
    Mode,
};

pub struct ViewBox {
    // The topmost row of the buffer being displayed
    // Zero-indexed
    top: usize,
    // The height in rows of the entire view box (minus the status bar)
    height: usize,
    // The leftmost row of the buffer being displayed
    left: usize,
    // The width in rows of the entire view box
    width: usize,
}

impl ViewBox {
    pub fn new(cols: u16, rows: u16) -> Self {
        ViewBox {
            top: 0,
            height: rows as usize - 1, // Reserve one for the status bar
            left: 0,
            width: cols as usize,
        }
    }

    pub fn adjust(&mut self, buffer: &Buffer) {
        let col = buffer.get_col();
        let row = buffer.get_row();

        if self.top < row {
            self.top = row;
        } else if self.top + self.height < row {
            self.top = row - self.height;
        }

        if self.left > col {
            self.left = col;
        } else if self.left + self.width < col {
            self.left = col - self.width;
        }
    }

    pub fn flush(
        &self,
        buffer: &Buffer,
        status_bar: &StatusBar,
        mode: &Mode,
        path: &Option<PathBuf>,
    ) -> Result<()> {
        let mut stdout = stdout();

        let col = buffer.get_col();
        let row = buffer.get_row();
        let string = buffer.to_string();
        let lines = string.lines().skip(self.top).take(self.height);

        execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
        log(format!("In flush:"));
        lines.for_each(|line| {
            // TODO Check if this really cuts down the line correctly
            log(format!("line: {}", line));
            let line = &line[self.left..usize::min(self.width, line.len())];
            execute!(stdout, Print(line));
            execute!(stdout, MoveDown(1));
            execute!(stdout, MoveToColumn(0));
        });
        let status_message = match (mode, path) {
            (Mode::Command, _) => status_bar.buffer(),
            (Mode::Normal, Some(path)) => format!("Editing File: \"{}\"", path.to_string_lossy()),
            (Mode::Normal, None) => "".into(),
            (Mode::Insert, _) => "-- INSERT --".into(),
            (Mode::Visual, _) => "-- VISUAL --".into(),
        };

        execute!(
            stdout,
            MoveToRow(self.height as u16 + 1),
            Print(status_message)
        );

        let (new_col, new_row) = match mode {
            Mode::Command => (status_bar.idx() as u16, (self.height + 1) as u16),
            _ => (col as u16, row as u16),
        };
        execute!(stdout, MoveToColumn(new_col), MoveToRow(new_row));
        stdout.flush()?;

        Ok(())
    }
}
