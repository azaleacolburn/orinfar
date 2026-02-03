use crate::{
    DEBUG, buffer::Buffer, io::try_get_git_hash, log, mode::Mode, status_bar::StatusBar,
    undo::UndoTree, view_box::ViewBox,
};
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
use ropey::Rope;
use std::{
    io::{Write, stdout},
    path::PathBuf,
};

#[derive(Debug)]
pub struct View {
    boxes: Vec<ViewBox>,
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

        let status_message = match (mode, self.get_path()) {
            (Mode::Meta | Mode::Search, _) => status_bar.buffer(),
            (Mode::Normal, Some(path)) => {
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
        let (new_col, new_row) = if matches!(mode, Mode::Meta | Mode::Search) {
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
        let predicate =
            |view_box: &ViewBox| -> bool { view_box.x == x && view_box.y + view_box.height == y };

        self.position_of_box(predicate)
    }

    pub fn position_view_box_left(&mut self) -> Option<usize> {
        let view_box = self.get_view_box();

        let (x, y) = (view_box.x, view_box.y);
        let predicate =
            |view_box: &ViewBox| -> bool { view_box.y == y && view_box.x + view_box.width == x };

        self.position_of_box(predicate)
    }

    pub fn position_view_box_right(&mut self) -> Option<usize> {
        let view_box = self.get_view_box();

        let (x, y) = view_box.get_upper_right();
        let predicate = |view_box: &ViewBox| -> bool { view_box.y == y && view_box.x == x };

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

        let mut new_view_box = ViewBox::new(half_width, view_box.height, half_x, view_box.y);

        let original_width = view_box.width;

        view_box.width = half_width;
        if !original_width.is_multiple_of(2) {
            new_view_box.width += 1;
        }

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

        let buffer = self.get_buffer_mut();
        buffer.replace_contents(str, undo_tree);

        self.cursor = anchor;
    }

    pub fn load_file(&mut self) -> Result<()> {
        if let Some(path) = self.get_path().cloned() {
            let buffer = self.get_buffer_mut();
            if !std::fs::exists(&path)? {
                std::fs::write(path, buffer.rope.to_string())?;
                return Ok(());
            }

            let contents = std::fs::read_to_string(path)?;
            buffer.rope = Rope::from(contents);

            buffer.lines_for_updating = (0..buffer.len()).map(|_| true).collect::<Vec<bool>>();
            buffer.cursor = usize::min(buffer.cursor, buffer.rope.len_chars());
            buffer.has_changed = true;
        }

        Ok(())
    }

    pub fn write(&self) -> Result<()> {
        let buffer = self.get_buffer().to_string();
        if let Some(path) = self.get_path() {
            std::fs::write(path, buffer)?;
        } else {
            log!("WARNING: Cannot Write Unattached Buffer");
        }

        Ok(())
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

    pub fn set_path(&mut self, path: Option<PathBuf>) {
        let git_hash = try_get_git_hash(path.as_ref());
        let view_box = &mut self.boxes[self.cursor];

        view_box.git_hash = git_hash;
        view_box.path = path;
    }

    pub fn get_path(&self) -> Option<&PathBuf> {
        let view_box = &self.boxes[self.cursor];

        view_box.path.as_ref()
    }

    pub fn get_git_hash(&self) -> Option<&str> {
        let view_box = &self.boxes[self.cursor];

        view_box.git_hash.as_deref()
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
