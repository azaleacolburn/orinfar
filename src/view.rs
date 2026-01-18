use std::io::{Write, stdout};

use crate::{
    buffer::Buffer,
    mode::Mode,
    register::{self, RegisterHandler},
    status_bar::{self, StatusBar},
    utility::ranges_overlap,
    view,
    view_box::ViewBox,
};
use anyhow::Result;
use crossterm::{
    cursor::{MoveTo, MoveToColumn, MoveToRow, Show},
    execute,
    style::{Color, Print, SetForegroundColor},
    terminal::{Clear, ClearType},
};

pub struct View {
    boxes: Vec<ViewBox>,
    // represents which index of the view box the cursor is in
    cursor: usize,
    width: u16,
    height: u16,
}

impl View {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            boxes: vec![ViewBox::new(cols, rows, 0, 0)],
            cursor: 0,
            width: cols, // Don't subtract one because each viewbox handles line nums separately
            height: rows - 1,
        }
    }

    pub fn get_buffer(&mut self) -> &mut Buffer {
        &mut self.boxes[self.cursor].buffer
    }

    pub fn get_view_box(&mut self) -> &mut ViewBox {
        &mut self.boxes[self.cursor]
    }

    pub fn flush(
        &self,
        status_bar: &StatusBar,
        mode: &Mode,
        chained: &[char],
        count: u16,
        register: char,
        path: Option<&std::path::PathBuf>,
        git_hash: Option<&str>,
        adjusted: bool,
    ) -> Result<()> {
        self.boxes.iter().for_each(|f| {
            f.flush(adjusted);
        });

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
                    format!("\"{register}")
                };
                let chained_str = chained.iter().collect::<String>();

                let git_hash = git_hash.unwrap_or("");

                let middle_buffer = (0..(self.width as usize)
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
                    format!("\"{register}")
                };
                let chained_str = chained.iter().collect::<String>();

                format!("{info_str}{count_str}{reg_str}{chained_str}")
            }
            (Mode::Insert, _) => "-- INSERT --".into(),
            (Mode::Visual, _) => "-- VISUAL --".into(),
        };
        execute!(
            stdout(),
            SetForegroundColor(Color::White),
            MoveTo(0, self.height + 1),
            Clear(ClearType::CurrentLine),
            Print(status_message)
        )?;

        // TODO Figure out what was going on here
        let (new_col, new_row) = if matches!(mode, Mode::Meta) {
            (status_bar.idx(), self.height + 1)
        } else {
            let view_box = &self.boxes[self.cursor];
            view_box.new_cursor_position()
        };
        execute!(stdout(), MoveToColumn(new_col), MoveToRow(new_row), Show)?;
        stdout().flush()?;

        Ok(())
    }

    pub fn add_view_box(&mut self, x: u16, y: u16, height: u16, width: u16) {
        let new_view_box = ViewBox::new(width, height, x, y);
        let horizontal_new = x..x + width;
        let vertical_new = y..y + height;

        self.boxes
            .iter_mut()
            .filter(|view_box| {
                let horizontal_old = view_box.x..view_box.x + view_box.width();
                let vertical_old = view_box.y..view_box.y + view_box.height();

                ranges_overlap(&horizontal_new, &horizontal_old)
                    && ranges_overlap(&vertical_new, &vertical_old)
            })
            .for_each(|view_box| {
                if view_box.x <= new_view_box.x {
                    view_box.x = new_view_box.x + new_view_box.width()
                } else if view_box.x > new_view_box.x {
                    view_box.x = new_view_box.x + new_view_box.width()
                }
            });

        self.boxes.push(new_view_box);
    }
}
