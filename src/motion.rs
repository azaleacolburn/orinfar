use crate::{
    buffer::Buffer,
    utility::{is_symbol, on_next_input},
};
use crossterm::event::KeyCode;

pub struct Motion {
    pub name: char,
    command: fn(buffer: &mut Buffer),
    pub inclusive: bool,
}

impl Motion {
    pub fn exclusive(name: char, command: fn(buffer: &mut Buffer)) -> Self {
        Self {
            name,
            command,
            inclusive: false,
        }
    }

    pub fn inclusive(name: char, command: fn(buffer: &mut Buffer)) -> Self {
        Self {
            name,
            command,
            inclusive: true,
        }
    }

    // Called when the motion should be applied directly
    pub fn apply(&self, buffer: &mut Buffer) {
        (self.command)(buffer);
    }

    // Called when the motion is chained to an operator
    // Doesn't apply the motion to the buffer but returns where the motion would have gone
    pub fn evaluate(&self, buffer: &Buffer) -> usize {
        let mut fake_buffer = buffer.clone();
        (self.command)(&mut fake_buffer);

        fake_buffer.cursor
    }
}

// Word Manipulation
impl Buffer {
    pub fn word(buffer: &mut Self) {
        if buffer.get_curr_line().len_chars() == buffer.get_col() {
            return;
        }
        let mut c = buffer.get_curr_char();

        let last_legal_char = buffer.rope.len_chars() - 1;

        if is_symbol(c) {
            while is_symbol(c) && buffer.cursor < last_legal_char {
                c = unwrap_or_break!(buffer.next_and_char());
            }
        } else if c.is_alphanumeric() || c == '_' {
            while (c.is_alphanumeric() || c == '_') && buffer.cursor < last_legal_char {
                c = unwrap_or_break!(buffer.next_and_char());
            }
        }

        while c.is_whitespace() && buffer.cursor < last_legal_char {
            if c == '\n' {
                if buffer.cursor < last_legal_char {
                    buffer.cursor += 1;
                }
                return;
            }
            c = unwrap_or_break!(buffer.next_and_char());
        }
    }

    pub fn back(buffer: &mut Self) {
        if buffer.cursor == 0 {
            return;
        }
        let mut prev_char = unwrap_or_return!(buffer.get_prev_char());

        if prev_char == '\n' && buffer.cursor > 0 {
            buffer.cursor -= 1;
            return;
        }

        if is_symbol(prev_char) {
            while is_symbol(prev_char) && buffer.cursor > 0 {
                buffer.prev_char();
                prev_char = unwrap_or_break!(buffer.get_prev_char());
            }
        } else if prev_char.is_alphanumeric() || prev_char == '_' {
            while prev_char.is_alphanumeric() || prev_char == '_' && buffer.cursor > 0 {
                buffer.prev_char();
                prev_char = unwrap_or_break!(buffer.get_prev_char());
            }
        } else {
            while prev_char.is_whitespace() && buffer.cursor > 0 {
                buffer.prev_char();
                prev_char = unwrap_or_break!(buffer.get_prev_char());
            }

            if prev_char.is_alphanumeric() {
                while prev_char.is_alphanumeric() && buffer.cursor > 0 {
                    buffer.prev_char();
                    prev_char = unwrap_or_break!(buffer.get_prev_char());
                }
            } else if is_symbol(prev_char) {
                while is_symbol(prev_char) && buffer.cursor > 0 {
                    buffer.prev_char();
                    prev_char = unwrap_or_break!(buffer.get_prev_char());
                }
            }
        }
    }

    pub fn end_of_word(buffer: &mut Self) {
        if buffer.get_curr_line().len_chars() == buffer.get_col() {
            return;
        }

        let last_legal_char = buffer.rope.len_chars() - 1;
        let mut next_char = unwrap_or_return!(buffer.get_next_char());

        if next_char == '\n' {
            if buffer.cursor < last_legal_char {
                buffer.cursor += 2;
            }
            return;
        }

        if is_symbol(next_char) {
            while is_symbol(next_char) && buffer.cursor < last_legal_char {
                buffer.next_char();
                next_char = unwrap_or_break!(buffer.get_next_char());
            }
        } else if next_char.is_alphanumeric() || next_char == '_' {
            while next_char.is_alphanumeric() || next_char == '_' && buffer.cursor < last_legal_char
            {
                buffer.next_char();
                next_char = unwrap_or_break!(buffer.get_next_char());
            }
        } else {
            while next_char.is_whitespace() && buffer.cursor < last_legal_char {
                buffer.next_char();
                next_char = unwrap_or_break!(buffer.get_next_char());
            }
            if next_char.is_alphanumeric() {
                while next_char.is_alphanumeric() && buffer.cursor < last_legal_char {
                    buffer.next_char();
                    next_char = unwrap_or_break!(buffer.get_next_char());
                }
            } else if is_symbol(next_char) {
                while is_symbol(next_char) && buffer.cursor < last_legal_char {
                    buffer.next_char();
                    next_char = unwrap_or_break!(buffer.get_next_char());
                }
            }
        }
    }
}

// Find Single Characters
impl Buffer {
    fn find_generic(&mut self, key: KeyCode, traverse: impl Fn(&Self, char) -> Option<usize>) {
        if let KeyCode::Char(target) = key
            && let Some(position) = traverse(self, target)
        {
            self.cursor = position;
        }
    }

    pub fn find(buffer: &mut Self) {
        let find_forward =
            |key: KeyCode, buffer: &mut Self| buffer.find_generic(key, Self::find_next);

        on_next_input(buffer, find_forward).expect("Failed to get character to find");
    }

    pub fn find_until(buffer: &mut Self) {
        let find_until = |key: KeyCode, buffer: &mut Self| {
            buffer.find_generic(key, Self::find_next);
            if buffer.cursor != 0 {
                buffer.cursor -= 1;
            }
        };

        on_next_input(buffer, find_until).expect("Failed to get character to find");
    }

    pub fn find_back(buffer: &mut Self) {
        let find_back = |key: KeyCode, buffer: &mut Self| buffer.find_generic(key, Self::find_prev);

        on_next_input(buffer, find_back).expect("Failed to get character to find");
    }
}

// Find Matching
impl Buffer {
    /// Moves `self.cursor` to the next occurence of any character in `list` on the current line.
    /// Does not modify `self` if no such character is present
    ///
    /// # Returns
    /// - `Some` containing The character in `list` that was found first
    ///   (equivalent to `self.get_curr_char()` after calling this function)
    /// - `None` if no such character was found
    fn find_from_list_on_line(&mut self, list: &[char]) -> Option<char> {
        let anchor = self.cursor;
        let end_of_line = self.get_end_of_line();

        let mut c = self.get_curr_char();

        while !list.contains(&c) && end_of_line > self.cursor + 1 {
            self.cursor += 1;
            c = self.get_curr_char();
        }

        if list.contains(&c) {
            return Some(c);
        }

        self.cursor = anchor;
        None
    }

    /// Moves `self.cursor` the matching occurence of `search_character`
    /// A matching occurence is one where the no unmatched occurences of `start_character` have
    /// occured since this occurence of `search_character`.
    ///
    /// Matching parentheses are the most obvious example. In the string "(())", index `0` matches
    /// with index `3`, even though index `2` is an earlier occurence of its corresponding
    /// character.
    ///
    /// # Arguments
    /// - `direction` must be `-1` or `1`
    fn search_matching(
        &mut self,
        start_character: char,
        search_character: char,
        direction: i32,
        exit_condition: impl Fn(usize) -> bool,
        anchor: usize,
    ) {
        let mut count = 0;
        let mut c = self.get_curr_char();
        if c != start_character {
            return;
        }

        loop {
            if exit_condition(self.cursor) {
                self.cursor = anchor;
                return;
            }

            self.cursor = usize::try_from(i32::try_from(self.cursor).unwrap() + direction).unwrap();
            c = self.get_curr_char();

            if c == start_character {
                count += 1;
            } else if c == search_character {
                if count == 0 {
                    return;
                }
                count -= 1;
            }
        }
    }

    /// Goes to the opposite bracket corresponding to the next bracket in the line (inclusive with the
    /// current character).
    pub fn next_corresponding_bracket(&mut self) {
        let anchor = self.cursor;
        let Some(c) = self.find_from_list_on_line(&['{', '}', '[', ']', '(', ')']) else {
            return;
        };

        let end_of_file = self.rope.len_chars();
        let start_character = c;
        let (search_character, direction): (char, i32) = match c {
            '{' => ('}', 1),
            '[' => (']', 1),
            '(' => (')', 1),
            '}' => ('{', -1),
            ']' => ('[', -1),
            ')' => ('(', -1),
            _ => panic!("Bug in next_bracket function"),
        };

        let forward_check = |cursor: usize| -> bool { end_of_file == cursor + 1 };
        let backward_check = |cursor: usize| -> bool { cursor == 0 };

        // NOTE
        // The only argument that changes is the `exit_condition`
        // We must have separate calls to avoid boxing the closures
        match direction {
            1 => self.search_matching(
                start_character,
                search_character,
                direction,
                forward_check,
                anchor,
            ),
            -1 => self.search_matching(
                start_character,
                search_character,
                direction,
                backward_check,
                anchor,
            ),
            _ => unreachable!(),
        }
    }
}

// Newline
impl Buffer {
    fn generic_newline(&mut self, is_at_end: impl Fn(&Self) -> bool, traverse: impl Fn(&mut Self)) {
        while self.is_empty_line() {
            if is_at_end(self) {
                return;
            }
            traverse(self);
        }

        while !self.is_empty_line() {
            if is_at_end(self) {
                return;
            }
            traverse(self);
        }
    }

    /// Moves the cursor to the next empty line after a non-empty line
    pub fn next_newline(&mut self) {
        self.generic_newline(Self::is_last_row, Self::next_row);
    }

    /// Moves the cursor to the next empty line after a non-empty line
    pub fn prev_newline(&mut self) {
        self.generic_newline(Self::is_first_row, Self::prev_row);
    }
}

// Misc
impl Buffer {
    pub fn beginning_of_line(buffer: &mut Self) {
        buffer.set_col(buffer.first_non_whitespace_col());
    }
}
