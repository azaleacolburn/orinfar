use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveDown, MoveTo, MoveToColumn, MoveToRow, Show},
    queue,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use ropey::RopeSlice;
use std::{
    io::{StdoutLock, Write, stdout},
    path::PathBuf,
};

use crate::{
    global_state::GlobalState,
    highlight::{HLBlock, HLEnd},
    mode::Mode,
    status_bar::StatusBar,
    view::View,
    view_box::ViewBox,
};

impl View {
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

    pub fn render(&self, global_state: &GlobalState, adjusted: bool) -> Result<()> {
        let register = global_state.register_handler.get_curr_reg();

        let mut errors = self.renderable_boxes().filter_map(|(i, view_box)| {
            let adjusted = adjusted && i == self.cursor;
            view_box.render(adjusted).err()
        });
        if let Some(err) = errors.next() {
            return Err(err);
        }

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
            view_box.cursor_position()
        };
        queue!(stdout, MoveToColumn(new_col), MoveToRow(new_row), Show)?;

        stdout.flush()?;
        Ok(())
    }
}

impl ViewBox {
    fn write_buffer(&self, stdout: &mut StdoutLock, left_padding: usize) -> Result<()> {
        let lines = self
            .buffer
            .rope
            .lines()
            .zip(self.buffer.lines_for_updating.iter())
            .enumerate()
            .skip(self.top)
            .take(self.height.into());

        queue!(stdout, Hide, MoveTo(self.x, self.y))?;
        let mut padding_buffer = String::with_capacity(left_padding);

        let clear_str: String = (0..=self.width).map(|_| ' ').collect();

        // NOTE
        // Taking the length to avoid having to clone all of lines
        // Since its used in the statement below
        let maybe_len_lines = u16::try_from(lines.len()).ok();

        if let Some(_tree) = self.parse_tree.as_ref()
            && let Some((_parser, language)) = self.parser.as_ref()
            && let Some(path) = self.path()
            && let Some(ex) = path.extension()
            && let Some(ex) = ex.to_str()
            && language.extensions.contains(&ex.to_string())
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
            queue!(stdout, MoveTo(self.x, self.y + len_lines))?;

            (len_lines..self.height).for_each(|_| {
                queue!(stdout, Print(&clear_str), MoveDown(1), MoveToColumn(self.x))
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
                queue!(stdout, MoveDown(1)).expect("Crossterm MoveDown command failed");
                return;
            }

            Self::clear_line(clear_str, stdout);
            self.print_padding(padding_buffer, left_padding, line_num, stdout);

            let line_len = Self::calculate_total_line_len(line);
            if line_len == 0 {
                queue!(stdout, MoveToColumn(self.x), MoveDown(1))
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
            queue!(
                stdout,
                SetForegroundColor(hl.fg_color),
                SetBackgroundColor(hl.bg_color),
                Print(text)
            )
            .expect("Crossterm print hl block command failed");
        }

        queue!(stdout, MoveToColumn(self.x), MoveDown(1)).expect("Crossterm reset command failed");
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
                queue!(stdout, MoveDown(1)).expect("Crossterm MoveDown command failed");
                return;
            }

            Self::clear_line(clear_str, stdout);
            self.print_padding(padding_buffer, left_padding, line_num, stdout);

            let line_len = Self::calculate_total_line_len(line);
            if line_len == 0 {
                queue!(stdout, MoveToColumn(self.x), MoveDown(1))
                    .expect("Crossterm padding buffer print failed");
                return;
            }

            let line = self.slice_line(line, line_len);

            queue!(
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
        queue!(stdout, Print(&clear_str)).unwrap();
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

        queue!(
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

    pub fn render(&self, adjusted: bool) -> Result<()> {
        let mut stdout = stdout().lock();
        let left_padding = self.left_padding();

        if self.buffer.has_changed || adjusted {
            self.write_buffer(&mut stdout, left_padding)?;
        }

        Ok(())
    }
}
