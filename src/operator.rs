use crate::{buffer::Buffer, motion::Motion, register::RegisterHandler, Cursor, Mode};

pub struct Operator {
    pub name: Vec<char>,
    command: fn(
        end: Cursor,
        buffer: &mut Buffer,
        register_handler: &mut RegisterHandler,
        mode: &mut Mode,
    ),
}

impl Operator {
    pub fn new(
        name: &str,
        command: fn(
            end: Cursor,
            buffer: &mut Buffer,
            register_handler: &mut RegisterHandler,
            mode: &mut Mode,
        ),
    ) -> Self {
        Self {
            name: name.chars().collect(),
            command,
        }
    }

    pub fn execute(
        &self,
        motion: &Motion,
        buffer: &mut Buffer,
        register_handler: &mut RegisterHandler,
        mode: &mut Mode,
    ) {
        let end = motion.evaluate(buffer);
        (self.command)(end, buffer, register_handler, mode);
    }
}

pub fn delete(
    end: Cursor,
    buffer: &mut Buffer,
    register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
) {
    // NOTE We can't actually know the distance between cursors
    // without traversing the buffer since lines are of arbitrary length
    let mut count: usize = 0;
    while buffer.cursor != end {
        register_handler.push_char(buffer.get_curr_char());
        count += 1;
    }
    (0..count).into_iter().for_each(|_| {
        buffer.delete_curr_char();
    });
}

pub fn yank(
    end: Cursor,
    buffer: &mut Buffer,
    register_handler: &mut RegisterHandler,
    mode: &mut Mode,
) {
}
