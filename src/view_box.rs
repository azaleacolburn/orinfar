use crate::{buffer::Buffer, log, status_bar::StatusBar, Mode};
use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveDown, MoveTo, MoveToColumn, MoveToRow, Show},
    execute,
    style::{Color, Print, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::{
    fmt::format,
    io::{stdout, Stdout, Write},
    path::PathBuf,
};

pub struct ViewBox {
    // The topmost row of the buffer being displayed (zero-indexed)
    top: usize,
    // The height in rows of the entire view box (minus the status bar)
    height: usize,
    // The leftmost row of the buffer being displayed (zero-indexed)
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
            width: cols as usize - 1,
        }
    }

    pub fn adjust(&mut self, buffer: &Buffer) -> bool {
        let col = buffer.get_col();
        let row = buffer.get_row();
        let mut adjusted = false;

        if self.top > row {
            self.top = row;
            adjusted = true;
        } else if self.top + self.height <= row {
            self.top = row - self.height + 1;
            adjusted = true;
        }

        if self.left > col {
            self.left = col;
            adjusted = true;
        } else if self.left + self.width < col {
            self.left = col - self.width;
            adjusted = true;
        }

        adjusted
    }

    fn write_buffer(
        &self,
        buffer: &Buffer,
        stdout: &mut Stdout,
        left_padding: usize,
    ) -> Result<()> {
        log(format!("buffer\n{}", buffer.to_string()));

        let lines = buffer
            .rope
            .lines()
            .enumerate()
            .skip(self.top)
            .take(self.height);

        execute!(stdout, Hide, MoveTo(0, 0), Clear(ClearType::All))?;
        let mut padding_buffer = String::with_capacity(left_padding);

        lines.for_each(|(i, line)| {
            let i_str = i.to_string();
            for _ in 0..left_padding - i_str.len() {
                padding_buffer.push(' ');
            }
            padding_buffer.push_str(&i_str);
            padding_buffer.push(' ');
            execute!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                Print(padding_buffer.clone()),
            );
            padding_buffer.clear();

            let len = line.len_chars();
            if len == 0 {
                return;
            }

            // We actually do want to cut off the newline here, hence the `- 1`
            let line_len = if line.get_char(line.len_chars() - 1).unwrap() == '\n' {
                len - 1
            } else {
                len
            };
            let last_col = usize::min(self.left + self.width, line_len);
            let line = &line.slice(self.left..last_col);

            execute!(
                stdout,
                SetForegroundColor(Color::Blue),
                Print(line),
                MoveDown(1),
                MoveToColumn(0)
            );
        });

        Ok(())
    }

    pub fn flush(
        &self,
        buffer: &Buffer,
        status_bar: &StatusBar,
        mode: &Mode,
        path: &Option<PathBuf>,
        adjusted: bool,
    ) -> Result<()> {
        let mut stdout = stdout();
        let left_padding = (self.top + self.height).to_string().len();

        if buffer.has_changed || adjusted {
            self.write_buffer(buffer, &mut stdout, left_padding)?;
        }

        let col = buffer.get_col();
        let row = buffer.get_row();

        let status_message = match (mode, path) {
            (Mode::Command, _) => status_bar.buffer(),
            (Mode::Normal, Some(path)) => format!(
                "Editing File: \"{}\" {}b",
                path.to_string_lossy(),
                std::fs::read(path)?.len()
            ),
            (Mode::Normal, None) => "".into(),
            (Mode::Insert, _) => "-- INSERT --".into(),
            (Mode::Visual, _) => "-- VISUAL --".into(),
        };
        execute!(
            stdout,
            MoveTo(1, self.height as u16 + 1),
            SetForegroundColor(Color::White),
            Print(status_message)
        );

        let (new_col, new_row) = match mode {
            Mode::Command => (status_bar.idx() as u16, (self.height + 1) as u16),
            _ => {
                let row = row - self.top;
                let col = col - self.left + left_padding + 1;
                (col as u16, row as u16)
            }
        };
        execute!(stdout, MoveToColumn(new_col), MoveToRow(new_row), Show)?;
        stdout.flush()?;

        Ok(())
    }
}
