use crate::{buffer::Buffer, mode::Mode, status_bar::StatusBar, view_box::ViewBox};
use anyhow::Result;
use crossterm::{
    cursor::{MoveTo, MoveToColumn, MoveToRow, SetCursorStyle, Show},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use std::{
    io::{Write, stdout},
    path::PathBuf,
};

pub struct View {
    pub boxes: Vec<ViewBox>,
    // Represents which index of the view box the cursor is in
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

    pub fn get_buffer_mut(&mut self) -> &mut Buffer {
        &mut self.boxes[self.cursor].buffer
    }

    pub fn get_buffer(&self) -> &Buffer {
        &self.boxes[self.cursor].buffer
    }

    pub fn get_view_box(&mut self) -> &mut ViewBox {
        &mut self.boxes[self.cursor]
    }

    pub fn normal_unattached_status(chained: &[char], count: u16, register: char) -> String {
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

    pub fn normal_attached_status(
        &self,
        path: &PathBuf,
        chained: &[char],
        count: u16,
        register: char,
    ) -> Result<String> {
        let info_str = "Editing File: ".to_string();
        let file_size = std::fs::read(path)?.len().to_string();
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

        let git_hash = self.get_git_hash().unwrap_or("");

        let middle_buffer = (0..(self.width as usize)
                    - info_str.len()
                    - path.len()
                    - 2 // For the 2 quotations
                    - 3 // For the 3 spaces
                    - 1 // For 'b'
                    - file_size.len()
                    - reg_str.len()
                    - count_str.len()
                    - chained_str.len()
                    - git_hash.len())
            .map(|_| ' ')
            .collect::<String>();

        Ok(format!(
            "{info_str}\"{path}\" {file_size}b {reg_str}{count_str} {chained_str}{middle_buffer}{git_hash}",
        ))
    }

    pub fn status_message(
        &self,
        status_bar: &StatusBar,
        mode: &Mode,
        chained: &[char],
        count: u16,
        register: char,
    ) -> Result<String> {
        let status_message = match (mode, self.get_path()) {
            (Mode::Meta | Mode::Search, _) => status_bar.buffer(),
            (Mode::Normal, Some(path)) => {
                self.normal_attached_status(path, chained, count, register)?
            }
            (Mode::Normal, None) => Self::normal_unattached_status(chained, count, register),
            (Mode::Insert, _) => "-- INSERT --".into(),
            (Mode::Visual, _) => "-- VISUAL --".into(),
        };

        Ok(status_message)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn flush(
        &self,
        status_bar: &StatusBar,
        mode: &Mode,
        chained: &[char],
        count: u16,
        register: char,
        adjusted: bool,
    ) -> Result<()> {
        let mut errors = self.boxes.iter().enumerate().filter_map(|(i, view_box)| {
            let adjusted = adjusted && i == self.cursor;
            view_box.flush(adjusted).err()
        });
        if let Some(err) = errors.next() {
            return Err(err);
        }

        let mut stdout = stdout().lock();

        let status_message = self.status_message(status_bar, mode, chained, count, register)?;
        execute!(
            stdout,
            SetForegroundColor(Color::White),
            MoveTo(0, self.height + 1),
            Clear(ClearType::CurrentLine),
            Print(status_message)
        )?;

        // TODO Figure out what was going on here
        let (new_col, new_row) = if matches!(mode, Mode::Meta | Mode::Search) {
            (status_bar.idx(), self.height + 1)
        } else {
            let view_box = &self.boxes[self.cursor];
            view_box.cursor_position()
        };
        execute!(stdout, MoveToColumn(new_col), MoveToRow(new_row), Show)?;

        stdout.flush()?;
        Ok(())
    }
}

/// `ViewBox` Manipulation Methods
impl View {
    pub fn position_of_box<P>(&self, predicate: P) -> Option<usize>
    where
        P: FnMut(&ViewBox) -> bool,
    {
        self.boxes.iter().position(predicate)
    }

    /// # Returns
    /// The position (in `self.boxes`) of one `view_box` down, if it exists
    pub fn vb_down(&mut self) -> Option<usize> {
        let view_box = self.get_view_box();

        let (x, y) = view_box.get_lower_left();
        let predicate = |view_box: &ViewBox| -> bool { view_box.x == x && view_box.y == y };

        self.position_of_box(predicate)
    }

    pub fn vb_up(&mut self) -> Option<usize> {
        let view_box = self.get_view_box();

        let (x, y) = (view_box.x, view_box.y);
        let predicate =
            |view_box: &ViewBox| -> bool { view_box.x == x && view_box.y + view_box.height == y };

        self.position_of_box(predicate)
    }

    pub fn vb_left(&mut self) -> Option<usize> {
        let view_box = self.get_view_box();

        let (x, y) = (view_box.x, view_box.y);
        let predicate =
            |view_box: &ViewBox| -> bool { view_box.y == y && view_box.x + view_box.width == x };

        self.position_of_box(predicate)
    }

    pub fn vb_right(&mut self) -> Option<usize> {
        let view_box = self.get_view_box();

        let (x, y) = view_box.get_upper_right();
        let predicate = |view_box: &ViewBox| -> bool { view_box.y == y && view_box.x == x };

        self.position_of_box(predicate)
    }

    pub fn delete_curr_view_box(&mut self) {
        let mut down = self.vb_down();
        let mut up = self.vb_up();

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

        if half_height == 1 {
            return;
        }

        let mut new_view_box = ViewBox::new(view_box.width, half_height, view_box.x, half_y);

        let original_height = view_box.height;

        view_box.height = half_height;
        if !original_height.is_multiple_of(2) {
            new_view_box.height += 1;
        }

        self.boxes.push(new_view_box);
    }

    pub fn split_view_box_horizontal(&mut self, idx: usize) {
        let view_box = &mut self.boxes[idx];

        let half_width = view_box.width / 2;
        let half_x = half_width + view_box.x;

        let left_padding = view_box.left_padding() as u16;
        if half_width < left_padding {
            return;
        }

        let mut new_view_box = ViewBox::new(half_width, view_box.height, half_x, view_box.y);

        let original_width = view_box.width;

        view_box.width = half_width;
        if !original_width.is_multiple_of(2) {
            new_view_box.width += 1;
        }

        self.boxes.push(new_view_box);
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

pub fn terminal_setup(rows: u16, cols: u16) -> Result<()> {
    let mut stdout = stdout().lock();

    execute!(
        stdout,
        EnterAlternateScreen,
        Clear(ClearType::All),
        MoveToRow(0),
        SetForegroundColor(Color::Blue),
    )?;

    // Fill entire screen with spaces with the background color
    for row in 0..rows {
        execute!(stdout, MoveTo(0, row), Print(" ".repeat(cols as usize)))?;
    }
    execute!(stdout, MoveTo(0, 0))?;
    for row in 0..rows {
        execute!(stdout, MoveTo(0, row), Print(" ".repeat(cols as usize)))?;
    }
    execute!(stdout, MoveTo(0, 0))?;
    enable_raw_mode()?;

    Ok(())
}
