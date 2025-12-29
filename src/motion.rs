use crossterm::event::KeyCode;

use crate::{buffer::Buffer, log, on_next_input_buffer_only};

pub struct Motion<'a> {
    pub name: &'a str,
    command: fn(buffer: &mut Buffer),
    pub inclusive: bool,
}

impl<'a> Motion<'a> {
    pub fn exclusive(name: &'a str, command: fn(buffer: &mut Buffer)) -> Self {
        Self {
            name,
            command,
            inclusive: false,
        }
    }

    pub fn inclusive(name: &'a str, command: fn(buffer: &mut Buffer)) -> Self {
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

        return fake_buffer.cursor;
    }
}

pub fn prev_char(buffer: &mut Buffer) {
    buffer.prev_char();
}

pub fn next_row(buffer: &mut Buffer) {
    if buffer.is_last_row() {
        return;
    }

    buffer.next_line();
}

pub fn prev_row(buffer: &mut Buffer) {
    if buffer.get_row() == 0 {
        return;
    }

    buffer.prev_line();

    let len = buffer.get_curr_line().len_chars();

    let col = if len > 0 {
        usize::min(buffer.get_col(), len - 1)
    } else {
        0
    };
    buffer.set_col(col)
}

pub fn next_char(buffer: &mut Buffer) {
    buffer.next_char();
}

pub fn word(buffer: &mut Buffer) {
    if buffer.get_curr_line().len_chars() == buffer.get_col() {
        return;
    }
    let mut c = buffer.get_curr_char();

    let last_legal_char = buffer.get_end_of_line();

    // This has to be `- 2` because we don't want to get rid of the trailing space
    if !c.is_alphanumeric() {
        while !c.is_alphanumeric() && buffer.cursor != last_legal_char {
            c = unwrap_or_break!(buffer.next_and_char());
        }
    } else {
        while c.is_alphanumeric() && buffer.cursor < last_legal_char {
            c = unwrap_or_break!(buffer.next_and_char());
        }
        while c.is_whitespace() && buffer.cursor < last_legal_char {
            c = unwrap_or_break!(buffer.next_and_char());
        }
    }
}

pub fn back(buffer: &mut Buffer) {
    if buffer.cursor == 0 {
        return;
    }
    let mut prev_char = unwrap_or_return!(buffer.get_prev_char());

    if !prev_char.is_alphanumeric() {
        while !prev_char.is_alphanumeric() {
            buffer.prev_char();
            prev_char = unwrap_or_break!(buffer.get_prev_char());
        }
        while prev_char.is_alphanumeric() {
            // Next char without wrapping lines, since newlines aren't counted
            if buffer.get_col() > 0 {
                buffer.prev_char();
            } else {
                break;
            }
            prev_char = unwrap_or_break!(buffer.get_prev_char());
        }
    } else {
        while prev_char.is_alphanumeric() {
            buffer.prev_char();
            prev_char = unwrap_or_break!(buffer.get_prev_char());
        }
    }
}

pub fn end_of_word(buffer: &mut Buffer) {
    if buffer.get_curr_line().len_chars() == buffer.get_col() {
        return;
    }

    let mut next_char = unwrap_or_return!(buffer.get_next_char());
    let last_legal_char = buffer.get_end_of_line();

    if !next_char.is_alphanumeric() {
        while !next_char.is_alphanumeric() && buffer.cursor != last_legal_char {
            next_char = unwrap_or_break!(buffer.next_and_char());
        }
        while next_char.is_alphanumeric() && buffer.cursor != last_legal_char {
            // Next char without wrapping lines, since newlines aren't counted
            if buffer.get_col() + 1 < buffer.rope.len_chars() {
                buffer.next_char();
            } else {
                break;
            }
            next_char = unwrap_or_break!(buffer.get_next_char());
        }
    } else {
        while next_char.is_alphanumeric() && buffer.cursor != last_legal_char {
            buffer.next_char();
            next_char = unwrap_or_break!(buffer.get_next_char());
        }
    }
}

pub fn end_of_line(buffer: &mut Buffer) {
    buffer.end_of_line();
}

pub fn beginning_of_line(buffer: &mut Buffer) {
    buffer.set_col(buffer.first_non_whitespace_col());
}

pub fn find(buffer: &mut Buffer) {
    fn find(key: KeyCode, buffer: &mut Buffer) {
        if let KeyCode::Char(target) = key {
            let anchor = buffer.cursor;
            loop {
                if buffer.get_curr_char() == target {
                    return;
                }
                if buffer.cursor == buffer.get_end_of_line() {
                    break;
                }
                buffer.cursor += 1;
            }

            buffer.cursor = anchor
        }
    }

    on_next_input_buffer_only(buffer, find).unwrap();
}

pub fn find_back(buffer: &mut Buffer) {
    fn find_back(key: KeyCode, buffer: &mut Buffer) {
        if let KeyCode::Char(target) = key {
            let anchor = buffer.cursor;
            loop {
                if buffer.get_curr_char() == target {
                    return;
                }
                if buffer.cursor == 0 {
                    break;
                }
                buffer.cursor -= 1;
            }

            buffer.cursor = anchor
        }
    }

    on_next_input_buffer_only(buffer, find_back).unwrap();
}

// Goes to the opposite bracket corresponding to the next bracket in the line (inclusive with  the
// current character).
pub fn next_corresponding_bracket(buffer: &mut Buffer) {
    let anchor = buffer.cursor;
    let end_of_line = buffer.get_end_of_line();

    let mut c = buffer.get_curr_char();

    while c != '{' && c != '[' && c != '(' && c != ')' && c != ']' && c != '}' {
        if end_of_line > buffer.cursor {
            buffer.cursor += 1;
            c = buffer.get_curr_char();
        } else {
            buffer.cursor = anchor;
            return;
        }
    }

    let end_of_file = buffer.rope.len_chars();
    let mut count = 0;
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

    // TODO
    // There's probably a better way to do this
    let condition: Box<dyn Fn(usize) -> bool> = match direction {
        1 => Box::new(|cursor: usize| end_of_file > cursor + 1),
        -1 => Box::new(|cursor: usize| cursor as i32 > 0),
        _ => panic!(),
    };

    loop {
        if condition(buffer.cursor) {
            buffer.cursor = (buffer.cursor as i32 + direction) as usize;
            c = buffer.get_curr_char();
        } else {
            buffer.cursor = anchor;
            return;
        }

        if c == start_character {
            count += 1;
        } else if c == search_character {
            if count == 0 {
                break;
            } else {
                count -= 1;
            }
        }
    }
}

/// Moves the cursor to the next empty line after a non-empty line
pub fn next_newline(buffer: &mut Buffer) {
    while buffer.is_empty_line() {
        if buffer.is_last_row() {
            return;
        }
        next_row(buffer);
    }

    while !buffer.is_empty_line() {
        if buffer.is_last_row() {
            return;
        }
        next_row(buffer);
    }
}

/// Moves the cursor to the next empty line after a non-empty line
pub fn prev_newline(buffer: &mut Buffer) {
    while buffer.is_empty_line() {
        if buffer.is_first_row() {
            return;
        }
        prev_row(buffer);
    }

    while !buffer.is_empty_line() {
        if buffer.is_first_row() {
            return;
        }
        prev_row(buffer);
    }
}
