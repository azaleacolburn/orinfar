use crate::{buffer::Buffer, Cursor};

pub struct Motion {
    name: Vec<char>,
    // Can't modify buffer.buff but can modify buffer.cursor
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
            buffer.next_char()
            next_char!(buffer, c);
        }
    } else {
        while c.is_alphanumeric() {
            next_char!(buffer, c);
        }
        while c.is_whitespace() {
            next_char!(buffer, c);
        }
    }
}
