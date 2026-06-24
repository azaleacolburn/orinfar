use crate::{buffer::Buffer, file_io::try_get_git_hash, language::OrinLanguage};
use std::path::PathBuf;
use tree_sitter::{Parser, Tree};

pub struct ViewBox {
    // Components inherant to the view box
    pub buffer: Buffer,
    path: Option<PathBuf>,
    pub git_hash: Option<String>,

    pub parser: Option<(Parser, OrinLanguage)>,
    pub parse_tree: Option<Tree>,

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
            parser: None,
            parse_tree: None,

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

        // This doesn't take into account the gutter
        let left_padding = self.left_padding();

        if self.left > col {
            self.left = col;
            adjusted = true;
        } else if self.left + (self.width as usize) < col + left_padding {
            self.left = col + left_padding - self.width as usize;
            adjusted = true;
        }

        if adjusted {
            self.buffer.update_list_set(.., true);
        }

        adjusted
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
        let buffer_col = self.buffer.get_col();
        let buffer_row = self.buffer.get_row();

        // NOTE
        // We can't be on a row number smaller than the top row being rendered
        // unless this function is called after the cursor has moved
        // but before `ViewBox::adjust` has been called
        //
        // Of course, the difference between the buffer row and the top row
        // can't be greater than the size of the screen, which for all screens
        // I know about, should be fewer rows tall than `u16::MAX`
        let absolute_row = self.y + u16::try_from(buffer_row - self.top).unwrap_or(0);
        let absolute_col = self.x
            + u16::min(
                u16::try_from(buffer_col - self.left + left_padding).unwrap_or(0),
                self.width,
            );
        (absolute_col, absolute_row)
    }

    pub fn set_path(&mut self, path: Option<PathBuf>) {
        if let Some(path) = &path
            && let Some(ext) = path.extension()
            && let Some(ext) = ext.to_str()
            && let Some(language) = OrinLanguage::from_ext(ext)
        {
            let mut parser = Parser::new();

            parser
                .set_language(&language.lang)
                .expect("Failed to load C parser");
            self.parser = Some((parser, language));
        }

        self.git_hash = try_get_git_hash(path.as_ref());
        self.path = path;
    }

    pub const fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }
}
