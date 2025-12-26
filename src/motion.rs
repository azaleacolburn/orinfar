use crossterm::event::KeyCode;

use crate::{buffer::Buffer, log, on_next_input_buffer_only};

pub struct Motion<'a> {
    pub name: &'a [char],
    command: fn(buffer: &mut Buffer),
}

impl<'a> Motion<'a> {
    pub fn new(name: &'a [char], command: fn(buffer: &mut Buffer)) -> Self {
        Self { name, command }
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

pub fn word(buffer: &mut Buffer) {
    if buffer.get_curr_line().len_chars() - 1 == buffer.cursor {
        return;
    }
    let mut c = buffer.get_curr_char();

    let last_legal_char = buffer.get_curr_line().len_chars() - 1;

    // This has to be `- 2` because we don't want to get rid of the trailing space
    if !c.is_alphanumeric() {
        while !c.is_alphanumeric() && buffer.cursor != last_legal_char {
            c = unwrap_or_break!(buffer.next_and_char());
        }
    } else {
        while c.is_alphanumeric() && buffer.cursor != last_legal_char {
            c = unwrap_or_break!(buffer.next_and_char());
        }
        while c.is_whitespace() && buffer.cursor != last_legal_char {
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
    let mut next_char = unwrap_or_return!(buffer.get_next_char());

    if !next_char.is_alphanumeric() {
        while !next_char.is_alphanumeric() {
            next_char = unwrap_or_break!(buffer.next_and_char());
        }
        while next_char.is_alphanumeric() {
            // Next char without wrapping lines, since newlines aren't counted
            if buffer.get_col() + 1 < buffer.rope.len_chars() {
                buffer.next_char();
            } else {
                break;
            }
            next_char = unwrap_or_break!(buffer.get_next_char());
        }
    } else {
        while next_char.is_alphanumeric() {
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
            if let Some(position) = buffer
                .get_curr_line()
                .chars()
                .skip(buffer.get_col())
                .position(|c| c == target)
            {
                buffer.set_col(position);
            }
        }
    }

    on_next_input_buffer_only(buffer, find).unwrap();
}
