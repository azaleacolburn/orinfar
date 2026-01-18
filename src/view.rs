use crate::{
    DEBUG, buffer::Buffer, log, mode::Mode, status_bar::StatusBar, undo::UndoTree,
    view_box::ViewBox,
};
use anyhow::Result;
use crossterm::{
    cursor::{MoveTo, MoveToColumn, MoveToRow, Show},
    execute,
    style::{Color, Print, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use ropey::iter;
use std::io::{Write, stdout};

#[derive(Debug)]
pub struct View {
    boxes: Vec<ViewBox>,
    // represents which index of the view box the cursor is in
    pub cursor: usize,
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
        log!("View Boxes: {:?}", self.boxes);
        let errors = self.boxes.iter().enumerate().filter_map(|(i, view_box)| {
            let adjusted = adjusted && i == self.cursor;
            view_box.flush(adjusted).err()
        });
        if let Some(err) = errors.last() {
            return Err(err);
        }

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
            view_box.cursor_position()
        };
        execute!(stdout(), MoveToColumn(new_col), MoveToRow(new_row), Show)?;
        stdout().flush()?;

        Ok(())
    }

    pub fn get_lower_right(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    pub fn find_box<P>(&self, predicate: P) -> Option<&ViewBox>
    where
        P: FnMut(&&ViewBox) -> bool,
    {
        self.boxes.iter().find(predicate)
    }

    pub fn position_of_box<P>(&self, predicate: P) -> Option<usize>
    where
        P: FnMut(&ViewBox) -> bool,
    {
        self.boxes.iter().position(predicate)
    }

    pub fn split_view_box_vertical(&mut self, idx: usize) {
        let view_box = &mut self.boxes[idx];
        let original_height = view_box.height;

        let half_height = view_box.height / 2;
        let half_y = half_height + view_box.y;

        let new_view_box = ViewBox::new(view_box.width, half_height, view_box.x, half_y);

        view_box.height /= 2;
        if original_height % 2 == 0 {
            view_box.height += 1;
        }

        self.boxes.push(new_view_box);
    }

    pub fn add_view_box_arbitrary(&mut self, x: u16, y: u16, height: u16, width: u16) {
        let new_view_box = ViewBox::new(width, height, x, y);
        // let horizontal_new = x..x + width;
        // let vertical_new = y..y + height;

        assert!(x + width < self.width && y + height < self.height);

        self.boxes
            .iter_mut()
            // .filter(|view_box| {
            //     let horizontal_old = view_box.x..view_box.x + view_box.width();
            //     let vertical_old = view_box.y..view_box.y + view_box.height();
            //
            //     ranges_overlap(&horizontal_new, &horizontal_old)
            //         && ranges_overlap(&vertical_new, &vertical_old)
            // })
            .for_each(|view_box| {
                if view_box.x == new_view_box.x && view_box.y == new_view_box.y {
                    view_box.x += new_view_box.width;
                    view_box.y += new_view_box.height;
                } else {
                    view_box.width = self.width - new_view_box.width;
                    view_box.height = self.height - new_view_box.height;
                }
            });

        self.boxes.push(new_view_box);
    }

    pub fn replace_buffer_contents(
        &mut self,
        contents: impl ToString,
        cursor: usize,
        undo_tree: &mut UndoTree,
    ) {
        let str = contents.to_string();

        let anchor = self.cursor;
        self.cursor = cursor;

        let buffer = self.get_buffer();
        buffer.replace_contents(str, undo_tree);

        self.cursor = anchor;
    }

    pub fn adjust(&mut self) -> bool {
        let view_box = self.get_view_box();
        view_box.adjust()
    }

    pub fn cursor_to_last(&mut self) {
        self.set_cursor(self.boxes_len() - 1);
    }

    pub fn boxes_len(&self) -> usize {
        self.boxes.len()
    }

    pub fn set_cursor(&mut self, new: usize) {
        self.cursor = new;
    }
}
