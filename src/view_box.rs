use crate::{DEBUG, buffer::Buffer, log};
use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveDown, MoveTo, MoveToColumn},
    execute,
    style::{Color, Print, SetForegroundColor},
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

        let len_lines = u16::try_from(lines.len()).unwrap();

        execute!(stdout, Hide, MoveTo(self.x, self.y))?;
        let mut padding_buffer = String::with_capacity(left_padding);

        let clear_str: String = (0..self.width).map(|_| ' ').collect();

        lines.for_each(|(line_num, (line, should_update))| {
            if !should_update {
                execute!(stdout, MoveDown(1)).expect("Crossterm MoveDown command failed");
                return;
            }

            let line_num = line_num.to_string();
            // `-1` for the last space character that gets pushed
            for _ in 0..left_padding - line_num.len() - 1 {
                padding_buffer.push(' ');
            }
            padding_buffer.push_str(&line_num);
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

            let mut total_line_len = line.len_chars();
            if total_line_len > 0
                && let Some(c) = line.get_char(total_line_len - 1)
                && c == '\n'
            {
                total_line_len -= 1;
            }

            if total_line_len == 0 {
                execute!(stdout, MoveToColumn(self.x), MoveDown(1))
                    .expect("Crossterm padding buffer print failed");
                return;
            }

            // Number of characters that we're able to display in the current line
            let display_line_len = self.width as usize - left_padding;

            // Last column in the buffer that's being rendered to the screen
            let last_col = usize::min(self.left + display_line_len, total_line_len);

            // NOTE
            // What was happening here waswhen we cropped the line
            // we also cropped the newline character at the
            // end, meaning that we never moved down to the next row!
            let line = if self.left >= last_col {
                String::new()
            } else {
                line.slice(self.left..last_col)
                    .to_string()
                    .trim_matches('\n')
                    .to_string()
            };

            log!("line:{}", line);

            execute!(
                stdout,
                SetForegroundColor(Color::Blue),
                Print(line),
                MoveToColumn(self.x),
                MoveDown(1)
            )
            .expect("Crossterm print line command failed");
        });

        // This is for clearing trailing lines that we missed
        if len_lines < self.height {
            execute!(stdout, MoveTo(self.x, self.y + len_lines))?;
            (len_lines..self.height).for_each(|_| {
                execute!(stdout, Print(&clear_str), MoveDown(1), MoveToColumn(self.x))
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

    pub fn left_padding(&self) -> usize {
        (self.top + self.height as usize).to_string().len() + 1
    }

    pub const fn _get_lower_right(&self) -> (u16, u16) {
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
        let col = self.x
            + u16::min(
                u16::try_from(col - self.left + left_padding).unwrap(),
                self.width,
            );
        (col, row)
    }
}
