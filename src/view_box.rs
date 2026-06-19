use crate::{
    buffer::Buffer,
    file_io::try_get_git_hash,
    highlight::{HLBlock, HLEnd},
    language::{self, OrinLanguage},
};
use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveDown, MoveTo, MoveToColumn},
    execute,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
};
use ropey::RopeSlice;
use std::{
    io::{StdoutLock, stdout},
    path::PathBuf,
};
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

    fn write_buffer(&self, stdout: &mut StdoutLock, left_padding: usize) -> Result<()> {
        let lines = self
            .buffer
            .rope
            .lines()
            .zip(self.buffer.lines_for_updating.iter())
            .enumerate()
            .skip(self.top)
            .take(self.height.into());

        execute!(stdout, Hide, MoveTo(self.x, self.y))?;
        let mut padding_buffer = String::with_capacity(left_padding);

        let clear_str: String = (0..=self.width).map(|_| ' ').collect();

        // NOTE
        // Taking the length to avoid having to clone all of lines
        // Since its used in the statement below
        let maybe_len_lines = u16::try_from(lines.len()).ok();

        if let Some(_tree) = self.parse_tree.as_ref()
            && let Some((_parser, language)) = self.parser.as_ref()
            && let Some(path) = self.path.as_ref()
            && let Some(ex) = path.extension()
            && language
                .extensions
                .contains(&ex.to_str().unwrap().to_string())
        {
            self.print_line_hl(
                lines,
                // NOTE
                // Expensive
                self.highlight(),
                stdout,
                &mut padding_buffer,
                left_padding,
                &clear_str,
            );
        } else {
            self.print_lines_colorless(
                lines,
                stdout,
                &mut padding_buffer,
                left_padding,
                &clear_str,
            );
        }

        // This is for clearing trailing lines that we missed
        if let Some(len_lines) = maybe_len_lines
            && len_lines < self.height
        {
            execute!(stdout, MoveTo(self.x, self.y + len_lines))?;

            (len_lines..self.height).for_each(|_| {
                execute!(stdout, Print(&clear_str), MoveDown(1), MoveToColumn(self.x))
                    .expect("Crossterm clearing trailing lines failed");
            });
        }

        Ok(())
    }

    /// At this point, the highlight groups should have been cropped to fit within the line
    fn print_line_hl<'b>(
        &self,
        lines: impl Iterator<Item = (usize, (RopeSlice<'b>, &'b bool))>,
        hl_lines: Vec<Vec<HLBlock>>,
        stdout: &mut StdoutLock,

        padding_buffer: &mut String,
        left_padding: usize,
        clear_str: &str,
    ) {
        let hl_lines = hl_lines.into_iter().skip(self.top).take(self.height.into());
        let lines = lines
            .zip(hl_lines)
            .map(|((line_num, (line, should_update)), hl_blocks)| {
                (line_num, line, hl_blocks, should_update)
            });

        lines.for_each(|(line_num, line, hl_blocks, should_update)| {
            if !should_update {
                execute!(stdout, MoveDown(1)).expect("Crossterm MoveDown command failed");
                return;
            }

            Self::clear_line(clear_str, stdout);
            self.print_padding(padding_buffer, left_padding, line_num, stdout);

            let line_len = Self::calculate_total_line_len(line);
            if line_len == 0 {
                execute!(stdout, MoveToColumn(self.x), MoveDown(1))
                    .expect("Crossterm padding buffer print failed");
                return;
            }

            // NOTE
            // We don't need to slice the string, we can just choose the hl blocks we  want to print
            let line = line.to_string();

            let last_col = self.last_col(left_padding, line_len);
            let hl_blocks = self.crop_hl_blocks(&hl_blocks, last_col, line_len);

            if hl_blocks.is_empty() {
                return;
            }

            self.print_hl_blocks(&hl_blocks, &line, stdout);
        });
    }

    // Returns a new list of hl blocks that only print the line from `[self.left, last_col)`
    //
    // - Eliminates hl blocks that don't overlap with the line at all
    // - Crops hl blocks (on both ends) that go past the ends of the line
    // - Leaves hl blocks that are fully within the line in tact
    fn crop_hl_blocks(
        &self,
        hl_blocks: &[HLBlock],
        last_col: usize,
        line_len: usize,
    ) -> Vec<HLBlock> {
        let mut hl_blocks: Vec<HLBlock> = hl_blocks
            .iter()
            .filter(|hl| {
                hl.start <= last_col
                    && match hl.end {
                        HLEnd::Bounded(end) => end >= self.left,
                        HLEnd::EndOfLine => line_len >= self.left,
                    }
            })
            .map(std::clone::Clone::clone)
            .collect();

        // If the only blocks are out of scope, we don't need to render them :D
        if hl_blocks.is_empty() {
            return vec![];
        }

        // Crop first block
        hl_blocks[0].start = usize::max(self.left, hl_blocks[0].start);

        // Crop last block
        if let Some(block) = hl_blocks.last_mut() {
            match block.end {
                HLEnd::Bounded(end) => block.end = HLEnd::Bounded(usize::min(last_col, end)),
                HLEnd::EndOfLine => block.end = HLEnd::Bounded(last_col),
            }
        }

        hl_blocks
    }

    /// Prints a line highlighted based on `hl_blocks`.
    /// The line has already been sliced to the correct size
    /// The hl blocks have already been cropped
    fn print_hl_blocks(&self, hl_blocks: &[HLBlock], line: &str, stdout: &mut StdoutLock) {
        for hl in hl_blocks {
            let text = hl.slice_text(line);
            execute!(
                stdout,
                SetForegroundColor(hl.fg_color),
                SetBackgroundColor(hl.bg_color),
                Print(text)
            )
            .expect("Crossterm print hl block command failed");
        }

        execute!(stdout, MoveToColumn(self.x), MoveDown(1))
            .expect("Crossterm reset command failed");
    }

    fn print_lines_colorless<'b>(
        &self,
        lines: impl Iterator<Item = (usize, (RopeSlice<'b>, &'b bool))>,
        stdout: &mut StdoutLock,

        padding_buffer: &mut String,
        left_padding: usize,
        clear_str: &str,
    ) {
        lines.for_each(|(line_num, (line, should_update))| {
            if !should_update {
                execute!(stdout, MoveDown(1)).expect("Crossterm MoveDown command failed");
                return;
            }

            Self::clear_line(clear_str, stdout);
            self.print_padding(padding_buffer, left_padding, line_num, stdout);

            let line_len = Self::calculate_total_line_len(line);
            if line_len == 0 {
                execute!(stdout, MoveToColumn(self.x), MoveDown(1))
                    .expect("Crossterm padding buffer print failed");
                return;
            }

            let line = self.slice_line(line, line_len);

            execute!(
                stdout,
                SetForegroundColor(Color::Grey),
                Print(&line),
                MoveToColumn(self.x),
                MoveDown(1)
            )
            .expect("Crossterm print line command failed");
        });
    }

    fn clear_line(clear_str: &str, stdout: &mut StdoutLock) {
        execute!(stdout, Print(&clear_str)).unwrap();
    }

    fn print_padding(
        &self,
        padding_buffer: &mut String,
        left_padding: usize,
        line_num: usize,
        stdout: &mut StdoutLock,
    ) {
        let line_num = line_num.to_string();
        // `-1` for the last space character that gets pushed
        for _ in 0..left_padding - line_num.len() - 1 {
            padding_buffer.push(' ');
        }
        padding_buffer.push_str(&line_num);
        padding_buffer.push(' ');

        execute!(
            stdout,
            SetForegroundColor(Color::Grey),
            MoveToColumn(self.x),
            Print(padding_buffer.clone()),
        )
        .expect("Crossterm padding buffer print failed");
        padding_buffer.clear();
    }

    fn calculate_total_line_len(line: RopeSlice) -> usize {
        let mut total_line_len = line.len_chars();
        if total_line_len > 0
            && let Some(c) = line.get_char(total_line_len - 1)
            && c == '\n'
        {
            total_line_len -= 1;
        }

        total_line_len
    }

    /// Returns the last column in the line that's being rendered to the screen
    fn last_col(&self, left_padding: usize, line_len: usize) -> usize {
        // Number of characters that we're able to display in the current line
        let display_line_len = self.width as usize - left_padding;

        usize::min(self.left + display_line_len, line_len)
    }

    fn slice_line(&self, line: RopeSlice, last_col: usize) -> String {
        // NOTE
        // What was happening here was when we cropped the line
        // we also cropped the newline character at the
        // end, meaning that we never moved down to the next row!
        if self.left >= last_col {
            String::new()
        } else {
            line.slice(self.left..last_col)
                .to_string()
                .trim_matches('\n')
                .to_string()
        }
    }

    pub fn flush(&self, adjusted: bool) -> Result<()> {
        let mut stdout = stdout().lock();
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

    pub fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }
}
