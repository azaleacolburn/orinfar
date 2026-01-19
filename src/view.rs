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
            boxes: vec![ViewBox::new(cols, rows - 1, 0, 0)],
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

    #[allow(clippy::too_many_arguments)]
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
        let mut errors = self.boxes.iter().enumerate().filter_map(|(i, view_box)| {
            let adjusted = adjusted && i == self.cursor;
            view_box.flush(adjusted).err()
        });
        if let Some(err) = errors.next() {
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
}

impl View {
    pub const fn get_lower_right(&self) -> (u16, u16) {
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

    /// # Returns
    /// The position (in `self.boxes`) of one `view_box` down, if it exists
    pub fn position_view_box_down(&mut self) -> Option<usize> {
        let view_box = self.get_view_box();

        let (x, y) = view_box.get_lower_left();
        let predicate = |view_box: &ViewBox| -> bool { view_box.x == x && view_box.y == y };

        self.position_of_box(predicate)
    }

    pub fn position_view_box_up(&mut self) -> Option<usize> {
        let view_box = self.get_view_box();

        let (x, y) = (view_box.x, view_box.y);
        log!("here");
        let predicate = |view_box: &ViewBox| -> bool {
            log!(
                "vbx {} vby {} vbh {} y {} x {}",
                view_box.x,
                view_box.y,
                view_box.height,
                y,
                x
            );
            view_box.x == x && view_box.y + view_box.height == y
        };

        self.position_of_box(predicate)
    }

    pub fn position_view_box_left(&mut self) -> Option<usize> {
        let view_box = self.get_view_box();

        let (x, y) = (view_box.x, view_box.y);
        let predicate = |view_box: &ViewBox| -> bool {
            log!(
                "vbx {} vby {} vbh {} y {} x {}",
                view_box.x,
                view_box.y,
                view_box.height,
                y,
                x
            );
            view_box.y == y && view_box.x + view_box.width == x
        };

        self.position_of_box(predicate)
    }

    pub fn position_view_box_right(&mut self) -> Option<usize> {
        let view_box = self.get_view_box();

        let (x, y) = view_box.get_upper_right();
        let predicate = |view_box: &ViewBox| -> bool {
            log!(
                "vbx {} vby {} vbh {} y {} x {}",
                view_box.x,
                view_box.y,
                view_box.height,
                y,
                x
            );
            view_box.y == y && view_box.x == x
        };

        self.position_of_box(predicate)
    }

    pub fn delete_curr_view_box(&mut self) {
        let mut down = self.position_view_box_down();
        let mut up = self.position_view_box_up();

        let view_box = self.boxes.remove(self.cursor);
        if let Some(ref mut down) = down
            && *down > self.cursor
        {
            *down -= 1;
        }
        if let Some(ref mut up) = up
            && *up > self.cursor
        {
            *up -= 1;
        }

        self.cursor = usize::max(self.cursor, 1) - 1;

        match (down, up) {
            (_, Some(up_i)) => {
                let up_box = &mut self.boxes[up_i];
                up_box.height += view_box.height;
                self.cursor = up_i;
            }
            (Some(down_i), None) => {
                let down_box = &mut self.boxes[down_i];
                down_box.y = view_box.y;
                down_box.height += view_box.height;
                self.cursor = down_i;
            }
            (None, None) => {}
        }

        let view_box = self.get_view_box();
        view_box.buffer.has_changed = true;
    }

    pub fn split_view_box_vertical(&mut self, idx: usize) {
        let view_box = &mut self.boxes[idx];

        let half_height = view_box.height / 2;
        let half_y = half_height + view_box.y;

        let mut new_view_box = ViewBox::new(view_box.width, half_height, view_box.x, half_y);

        let original_height = view_box.height;
        log!(
            "half_height {} original_height {}",
            half_height,
            original_height
        );

        view_box.height = half_height;
        if !original_height.is_multiple_of(2) {
            new_view_box.height += 1;
        }
        log!("new height {}", new_view_box.height);

        self.boxes.push(new_view_box);
    }

    pub fn split_view_box_horizontal(&mut self, idx: usize) {
        let view_box = &mut self.boxes[idx];

        let half_width = view_box.width / 2;
        let half_x = half_width + view_box.x;

        let mut new_view_box = ViewBox::new(half_width, view_box.height, half_x, view_box.y);

        let original_width = view_box.width;
        log!(
            "half_width {} original_width {}",
            half_width,
            original_width
        );

        view_box.width = half_width;
        if !original_width.is_multiple_of(2) {
            new_view_box.width += 1;
        }
        log!("new width {}", new_view_box.width);

        self.boxes.push(new_view_box);
    }
}

impl View {
    pub fn replace_buffer_contents(
        &mut self,
        contents: &impl ToString,
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

    pub const fn cursor_to_last(&mut self) {
        self.set_cursor(self.boxes_len() - 1);
    }

    pub const fn boxes_len(&self) -> usize {
        self.boxes.len()
    }

    pub const fn set_cursor(&mut self, new: usize) {
        self.cursor = new;
    }
}
