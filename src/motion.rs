use crate::{buffer::Buffer, Cursor};

pub struct Motion {
    pub name: Vec<char>,
    command: fn(buffer: &mut Buffer),
}

impl Motion {
    pub fn new(name: &str, command: fn(buffer: &mut Buffer)) -> Self {
        Self {
            name: name.chars().collect(),
            command,
        }
    }

    // Called when the motion should be applied directly
    pub fn apply(&self, buffer: &mut Buffer) {
        (self.command)(buffer);
    }

    // Called when the motion is chained to an operator
    // Doesn't apply the motion to the buffer but returns where the motion would have gone
    pub fn evaluate(&self, buffer: &mut Buffer) -> Cursor {
        let mut fake_buffer = buffer.clone();
        (self.command)(&mut fake_buffer);

        return fake_buffer.cursor;
    }
}

pub fn word(buffer: &mut Buffer) {
    if buffer.is_empty_line() {
        return;
    }
    let mut c = buffer.get_curr_char();

    if !c.is_alphanumeric() {
        while !c.is_alphanumeric() {
            c = unwrap_or_break!(buffer.next_char());
        }
    } else {
        while c.is_alphanumeric() {
            c = unwrap_or_break!(buffer.next_char());
        }
        while c.is_whitespace() {
            c = unwrap_or_break!(buffer.next_char());
        }
    }
}

pub fn back(buffer: &mut Buffer) {
    if buffer.buff[buffer.cursor.row].is_empty() {
        return;
    }
    let mut c = buffer.get_curr_char();

    if !c.is_alphanumeric() {
        while !c.is_alphanumeric() {
            c = unwrap_or_break!(buffer.prev_char());
        }
    } else {
        while c.is_alphanumeric() {
            c = unwrap_or_break!(buffer.prev_char());
        }
        while c.is_whitespace() {
            c = unwrap_or_break!(buffer.prev_char());
        }
    }
}

pub fn end_of_word(buffer: &mut Buffer) {
    let mut next_char = unwrap_or_return!(buffer.get_next_char());

    if !next_char.is_alphanumeric() {
        while !next_char.is_alphanumeric() {
            next_char = unwrap_or_break!(buffer.next_char());
        }
        while next_char.is_alphanumeric() {
            // Next char without wrapping lines, since newlines aren't counted
            if buffer.cursor.col + 1 < buffer.buff[buffer.cursor.row].len() {
                buffer.cursor.col += 1;
            } else {
                break;
            }
            next_char = buffer.get_curr_char();
        }
    } else {
        while next_char.is_alphanumeric() {
            buffer.next_char();
            next_char = unwrap_or_break!(buffer.get_next_char());
        }
    }
}

pub fn end_of_line(buffer: &mut Buffer) {
    buffer.cursor.col = buffer.buff[buffer.cursor.row].len() - 1
}

pub fn beginning_of_line(buffer: &mut Buffer) {
    let first = buffer.buff[buffer.cursor.row]
        .iter()
        .position(|c| !c.is_whitespace())
        .unwrap_or(buffer.cursor.col);
    buffer.cursor.col = first
}
