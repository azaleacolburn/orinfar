use crate::{DEBUG, Mode, buffer::Buffer, log, status_bar::StatusBar};
use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveDown, MoveTo, MoveToColumn, MoveToRow, SetCursorStyle, Show},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use std::{
    io::{Stdout, Write, stdout},
    path::PathBuf,
};

pub struct ViewBox {
    // The topmost row of the buffer being displayed (zero-indexed)
    pub top: usize,
    // The height in rows of the entire view box (minus the status bar)
    height: usize,
    // The leftmost row of the buffer being displayed (zero-indexed)
    pub left: usize,
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

    pub fn adjust(&mut self, buffer: &mut Buffer) -> bool {
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

        if adjusted {
            buffer.update_list_set(.., true);
        }

        adjusted
    }

    fn write_buffer(
        &self,
        buffer: &Buffer,
        stdout: &mut Stdout,
        left_padding: usize,
    ) -> Result<()> {
        let lines = buffer
            .rope
            .lines()
            .zip(buffer.lines_for_updating.iter())
            .enumerate()
            .skip(self.top)
            .take(self.height);
        let len_lines = lines.len();

        execute!(stdout, Hide, MoveTo(0, 0))?;
        let mut padding_buffer = String::with_capacity(left_padding);

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
            execute!(
                stdout,
                Clear(ClearType::CurrentLine),
                SetForegroundColor(Color::DarkGrey),
                Print(padding_buffer.clone()),
            )
            .expect("Crossterm padding buffer print failed");
            padding_buffer.clear();

            let len = line.len_chars();
            if len == 0 {
                return;
            }

            let last_col = usize::min(self.left + self.width, len);
            log!(
                "eos {} len {} last_col {} left {}",
                self.left + self.width,
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
                MoveToColumn(0)
            )
            .expect("Crossterm print line command failed");
        });

        // This is for clearing trailing lines that we missed
        if len_lines < self.height {
            execute!(stdout, MoveTo(0, len_lines as u16))?;
            (len_lines..self.height).for_each(|_| {
                execute!(stdout, Clear(ClearType::CurrentLine), MoveDown(1))
                    .expect("Crossterm clearing trailing lines failed")
            });
        };

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn flush(
        &self,
        buffer: &Buffer,
        status_bar: &StatusBar,
        mode: &Mode,
        chained: &[char],
        count: u16,
        register: char,
        path: &Option<PathBuf>,
        git_hash: &Option<String>,
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
            (Mode::Meta, _) => status_bar.buffer(),
            (Mode::Normal, Some(path)) => {
                let info_str = "Editing File: ".to_string();
                let file_size = std::fs::read(path)?.len();
                let path = path.to_string_lossy();

                let count_str = if count == 1 {
                    String::new()
                } else {
                    count.to_string()
                };
                let reg_str = if register == '\"' {
                    String::new()
                } else {
                    format!("\"{}", register)
                };
                let chained_str = chained.iter().collect::<String>();

                let git_hash = git_hash.clone().unwrap_or_default();

                let middle_buffer = (0..self.width
                    - info_str.len()
                    - path.len()
                    - 2 // For the 2 quotations
                    - 3 // For the 3 spaces
                    - file_size.checked_ilog10().unwrap_or(1) as usize
                    - reg_str.len()
                    - count_str.len()
                    - chained_str.len()
                    - git_hash.len())
                    .map(|_| " ")
                    .collect::<String>();

                format!(
                    "{info_str}\"{path}\" {file_size}b {reg_str}{count_str} {chained_str}{middle_buffer}{git_hash}",
                )
            }
            (Mode::Normal, None) => {
                let info_str = "-- Unattached Buffer -- ".to_string();

                let count_str = if count == 1 {
                    String::new()
                } else {
                    count.to_string()
                };
                let reg_str = if register == '\"' {
                    String::new()
                } else {
                    format!("\"{}", register)
                };
                let chained_str = chained.iter().collect::<String>();

                format!("{info_str}{count_str}{reg_str}{chained_str}")
            }
            (Mode::Insert, _) => "-- INSERT --".into(),
            (Mode::Visual, _) => "-- VISUAL --".into(),
        };
        execute!(
            stdout,
            SetForegroundColor(Color::White),
            MoveTo(0, self.height as u16 + 1),
            Clear(ClearType::CurrentLine),
            Print(status_message)
        )?;

        let (new_col, new_row) = match mode {
            Mode::Meta => (status_bar.idx() as u16, (self.height + 1) as u16),
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

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn _width(&self) -> usize {
        self.width
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
