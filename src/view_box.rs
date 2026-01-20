use crate::{DEBUG, buffer::Buffer, log};
use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveDown, MoveTo, MoveToColumn, MoveToRow, SetCursorStyle},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use std::{
    io::{Stdout, stdout},
    path::PathBuf,
};

#[derive(Debug)]
pub struct ViewBox {
    // Components inherant to the view box
    pub buffer: Buffer,
    pub path: Option<PathBuf>,
    pub git_hash: Option<String>,

    // The x and y corrdinates of the upper right hand corner of where the buffer will be displayed
    pub x: u16,
    pub y: u16,
    // The topmost row of the buffer being displayed (zero-indexed)
    pub top: usize,
    // The height in rows of the entire view box (minus the status bar)
    pub height: u16,
    // The leftmost row of the buffer being displayed (zero-indexed)
    pub left: usize,
    // The width in rows of the entire view box
    pub width: u16,
}

impl ViewBox {
    /// # Arguments
    /// - cols: the number of cols this view box has
    /// - rows: the number of rows this view box has
    /// - x: the x position of the upper right hand corner of this view box
    /// - y: the y position of the upper right hand corner of this view box
    pub fn new(cols: u16, rows: u16, x: u16, y: u16) -> Self {
        Self {
            buffer: Buffer::new(),
            path: None,
            git_hash: None,

            x,
            y,
            top: 0,
            height: rows,
            left: 0,
            // Reserve one for line numbers
            // TODO
            // Have option to not have line nums
            width: cols - 1,
        }
    }

    pub fn adjust(&mut self) -> bool {
        let col = self.buffer.get_col();
        let row = self.buffer.get_row();
        let mut adjusted = false;

        if self.top > row {
            self.top = row;
            adjusted = true;
        } else if self.top + self.height as usize <= row {
            self.top = row - self.height as usize + 1;
            adjusted = true;
        }

        if self.left > col {
            self.left = col;
            adjusted = true;
        } else if self.left + (self.width as usize) < col {
            self.left = col - self.width as usize;
            adjusted = true;
        }

        if adjusted {
            self.buffer.update_list_set(.., true);
        }

        adjusted
    }

    fn write_buffer(&self, stdout: &mut Stdout, left_padding: usize) -> Result<()> {
        let lines = self
            .buffer
            .rope
            .lines()
            .zip(self.buffer.lines_for_updating.iter())
            .enumerate()
            .skip(self.top)
            .take(self.height.into());

        assert!(lines.len() <= self.height.into());
        #[allow(clippy::cast_possible_wrap)]
        let len_lines = u16::try_from(lines.len()).unwrap();

        execute!(stdout, Hide, MoveTo(self.x, self.y))?;
        let mut padding_buffer = String::with_capacity(left_padding);

        let clear_str: String = (0..self.width).map(|_| ' ').collect();

        lines.for_each(|(i, (line, should_update))| {
            if !should_update {
                execute!(stdout, MoveDown(1)).expect("Crossterm MoveDown command failed");
                return;
            }

            let i_str = i.to_string();
            for _ in 0..left_padding - i_str.len() {
                padding_buffer.push(' ');
            }
            padding_buffer.push_str(&i_str);
            padding_buffer.push(' ');

            execute!(stdout, Print(&clear_str)).unwrap();

            execute!(
                stdout,
                SetForegroundColor(Color::Grey),
                MoveToColumn(self.x),
                Print(padding_buffer.clone()),
            )
            .expect("Crossterm padding buffer print failed");
            padding_buffer.clear();

            let len = line.len_chars();
            if len == 0 {
                return;
            }

            let last_col = usize::min(self.left + self.width as usize, len);
            log!(
                "eos {} len {} last_col {} left {}",
                self.left + self.width as usize,
                len,
                last_col,
                self.left
            );
            let line = if self.left >= last_col {
                String::from("\n")
            } else {
                line.slice(self.left..last_col).to_string()
            };

            execute!(
                stdout,
                SetForegroundColor(Color::Blue),
                Print(line),
                MoveToColumn(self.x)
            )
            .expect("Crossterm print line command failed");
        });

        // This is for clearing trailing lines that we missed
        if len_lines < self.height {
            execute!(stdout, MoveTo(self.x, self.y + len_lines))?;
            (len_lines..self.height).for_each(|_| {
                execute!(stdout, MoveToColumn(self.x), Print(&clear_str), MoveDown(1))
                    .expect("Crossterm clearing trailing lines failed");
            });
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn flush(&self, adjusted: bool) -> Result<()> {
        let mut stdout = stdout();
        let left_padding = self.left_padding();

        if self.buffer.has_changed || adjusted {
            self.write_buffer(&mut stdout, left_padding)?;
        }

        Ok(())
    }

    pub fn clear_view_box_line(&self) -> Result<()> {
        let str: String = (0..self.width).map(|_| ' ').collect();
        execute!(stdout(), Print(str))?;

        Ok(())
    }

    pub fn left_padding(&self) -> usize {
        (self.top + self.height as usize).to_string().len()
    }

    pub const fn get_lower_right(&self) -> (u16, u16) {
        (self.x + self.width, self.y + self.height)
    }

    pub const fn get_lower_left(&self) -> (u16, u16) {
        (self.x, self.y + self.height)
    }

    pub const fn get_upper_right(&self) -> (u16, u16) {
        (self.x + self.width, self.y)
    }

    /// # Returns
    /// The current cursor position on the absolute screen
    /// Given that the cursor is in the given view box
    pub fn cursor_position(&self) -> (u16, u16) {
        let left_padding = self.left_padding();
        let col = self.buffer.get_col();
        let row = self.buffer.get_row();

        let row = self.y + u16::try_from(row - self.top).unwrap();
        let col = self.x + u16::try_from(col - self.left + left_padding + 1).unwrap();
        (col, row)
    }

    pub const fn buffer(&mut self) -> &mut Buffer {
        &mut self.buffer
    }
}

pub fn cleanup() -> Result<()> {
    disable_raw_mode()?;
    execute!(
        stdout(),
        ResetColor,
        Clear(ClearType::All),
        SetCursorStyle::SteadyBlock,
        LeaveAlternateScreen
    )?;

    Ok(())
}

pub fn setup(rows: u16, cols: u16) -> Result<()> {
    execute!(
        stdout(),
        EnterAlternateScreen,
        Clear(ClearType::All),
        MoveToRow(0),
        SetForegroundColor(Color::Blue),
    )?;

    // Fill entire screen with spaces with the background color
    for row in 0..rows {
        execute!(stdout(), MoveTo(0, row), Print(" ".repeat(cols as usize)))?;
    }
    execute!(stdout(), MoveTo(0, 0))?;
    for row in 0..rows {
        execute!(stdout(), MoveTo(0, row), Print(" ".repeat(cols as usize)))?;
    }
    execute!(stdout(), MoveTo(0, 0))?;
    enable_raw_mode()?;

    Ok(())
}
