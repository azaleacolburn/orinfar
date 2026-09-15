use crate::{
    buffer::Buffer,
    global_state::GlobalState,
    mode::Mode,
    status_bar::StatusBar,
    view_box::ViewBox,
    view_node::ViewNode::{self},
};
use anyhow::Result;
use crossterm::{
    cursor::{MoveTo, MoveToColumn, MoveToRow, SetCursorStyle, Show},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use std::{
    io::{Write, stdout},
    path::PathBuf,
    ptr,
};

/// Represents the entire view of the editor in the terminal
pub struct View {
    view_tree: Box<ViewNode>,
    current_view_box: *mut ViewNode,
    width: u16,
    height: u16,
}

impl View {
    pub fn new(cols: u16, rows: u16) -> Self {
        let mut boxed = Box::new(ViewNode::Leaf(ViewBox::new()));
        let ptr = ptr::from_mut(boxed.as_mut());

        Self {
            view_tree: boxed,
            current_view_box: ptr,
            width: cols, // Don't subtract one because each viewbox handles line nums separately
            height: rows - 1,
        }
    }

    pub fn get_buffer_mut(&mut self) -> &mut Buffer {
        // We originally had a mutable reference
        // But `Self::get_view_box` returns an immutable reference
        // So we just cast back to get our original reference
        // This should be perfectly safe
        let view_box = ptr::from_ref(self.get_view_box());
        let view_box_mut = unsafe { view_box.cast_mut().as_mut() }.unwrap();

        &mut view_box_mut.buffer
    }

    pub fn get_buffer(&self) -> &Buffer {
        &self.get_view_box().buffer
    }

    pub fn get_view_box_mut(&mut self) -> &mut ViewBox {
        let view_box = ptr::from_ref(self.get_view_box()) as *mut ViewBox;

        unsafe { view_box.as_mut().unwrap() }
    }

    /// Guaranteed to not mutatble `self`
    pub fn get_view_box(&self) -> &ViewBox {
        let view_node = unsafe {
            self.current_view_box
                .as_ref()
                .expect("Invalid Pointer to Current View Box: Bug In Orinfar")
        };

        match view_node {
            ViewNode::Leaf(b) => b,
            _ => panic!("Current View Box Not Leaf: Bug In Orinfar"),
        }
    }

    pub fn normal_unattached_status(chained: &[char], count: u32, register: char) -> String {
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
        count: u32,
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

        let status_bar_width: usize = info_str.len()
            + path.len()
            + 2
            + 3
            + 1
            + file_size.len()
            + reg_str.len()
            + count_str.len()
            + chained_str.len()
            + git_hash.len();

        if status_bar_width > self.width as usize {
            // TODO Maybe add more breakpoints???
            let abridged_size = info_str.len() + path.len() + 2 + 3 + 1 + file_size.len();
            if abridged_size > self.width as usize {
                return Ok(String::new());
            }

            return Ok(format!("{info_str}\"{path}\" {file_size}b"));
        }

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
        count: u32,
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

    pub fn render(
        &mut self,
        global_state: &GlobalState,
        adjusted: bool,
        resized: bool,
    ) -> Result<()> {
        let register = global_state.register_handler.get_curr_reg();

        self.view_tree
            .render_view_node(0, 0, self.height, self.width, adjusted, resized);

        let mut stdout = stdout().lock();

        let status_message = self.status_message(
            &global_state.status_bar,
            &global_state.mode,
            &global_state.chained,
            global_state.count,
            register,
        )?;

        queue!(
            stdout,
            SetForegroundColor(Color::White),
            MoveTo(0, self.height + 1),
            Clear(ClearType::CurrentLine),
            Print(status_message)
        )?;

        // TODO Figure out what was going on here
        let (new_col, new_row) = if matches!(global_state.mode, Mode::Meta | Mode::Search) {
            (global_state.status_bar.idx(), self.height + 1)
        } else {
            let view_box = &self.get_view_box();
            view_box.cursor_position(view_box.cached_render_info.as_ref().unwrap())
        };
        queue!(stdout, MoveToColumn(new_col), MoveToRow(new_row), Show)?;

        stdout.flush()?;
        Ok(())
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.width = cols;
        self.height = rows;
    }
}

/// `ViewBox` Manipulation Methods
// impl View {
//     /// # Returns
//     ///
//     /// The position (in `self.boxes`) of one `view_box` down, if it exists
//     pub fn path_to_view_box_down(&mut self) -> Option<usize> {
//         let view_box = self.get_view_box();
//
//         let (x, y) = view_box.get_lower_left();
//         let predicate = |view_box: &ViewBox| -> bool { view_box.x == x && view_box.y == y };
//
//         self.position_of_box(predicate)
//     }
//
//     pub fn position_view_box_up(&mut self) -> Option<usize> {
//         let view_box = self.get_view_box();
//
//         let (x, y) = (view_box.x, view_box.y);
//         let predicate =
//             |view_box: &ViewBox| -> bool { view_box.x == x && view_box.y + view_box.height == y };
//
//         self.position_of_box(predicate)
//     }
//
//     pub fn position_view_box_left(&mut self) -> Option<usize> {
//         let view_box = self.get_view_box();
//
//         let (x, y) = (view_box.x, view_box.y);
//         let predicate =
//             |view_box: &ViewBox| -> bool { view_box.y == y && view_box.x + view_box.width == x };
//
//         self.position_of_box(predicate)
//     }
//
//     pub fn position_view_box_right(&mut self) -> Option<usize> {
//         let view_box = self.get_view_box();
//
//         let (x, y) = view_box.get_upper_right();
//         let predicate = |view_box: &ViewBox| -> bool { view_box.y == y && view_box.x == x };
//
//         self.position_of_box(predicate)
//     }
//

impl View {
    pub fn delete_curr_view_box(&mut self) {}
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
