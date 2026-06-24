use crate::{buffer::Buffer, view_box::ViewBox};
use anyhow::Result;
use crossterm::{
    cursor::{MoveTo, MoveToRow, SetCursorStyle},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use std::io::stdout;

pub struct View {
    boxes: Vec<ViewBox>,
    should_render: Vec<bool>,
    // Represents which index of the view box the cursor is in
    pub cursor: usize,
    pub width: u16,
    pub height: u16,
}

impl View {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            boxes: vec![ViewBox::new(cols, rows - 1, 0, 0)],
            should_render: vec![true],
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

    pub fn get_view_box(&self) -> &ViewBox {
        &self.boxes[self.cursor]
    }

    pub fn get_view_box_mut(&mut self) -> &mut ViewBox {
        &mut self.boxes[self.cursor]
    }

    pub fn box_count(&self) -> usize {
        self.boxes.len()
    }

    pub fn renderable_boxes(&self) -> impl Iterator<Item = (usize, &ViewBox)> {
        self.boxes
            .iter()
            .enumerate()
            .filter(|(i, _)| self.should_render[*i])
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

        let _ = self.should_render.remove(self.cursor);
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

        let view_box = self.get_view_box_mut();
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
        self.should_render.push(true);
    }

    pub fn split_view_box_horizontal(&mut self, idx: usize) {
        let view_box = &mut self.boxes[idx];

        let half_width = view_box.width / 2;
        let half_x = half_width + view_box.x;

        let left_padding = view_box.left_padding();
        if (half_width as usize) < left_padding {
            return;
        }

        let mut new_view_box = ViewBox::new(half_width, view_box.height, half_x, view_box.y);

        let original_width = view_box.width;

        view_box.width = half_width;
        if !original_width.is_multiple_of(2) {
            new_view_box.width += 1;
        }

        self.boxes.push(new_view_box);
        self.should_render.push(true);
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
