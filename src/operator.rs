use crate::{buffer::Buffer, motion::Motion, register::RegisterHandler, Cursor, Mode};

pub struct Operator {
    pub name: Vec<char>,
    command: fn(
        motion: &Motion,
        buffer: &mut Buffer,
        register_handler: &mut RegisterHandler,
        mode: &mut Mode,
    ),
}

impl Operator {
    pub fn new(
        name: &str,
        command: fn(
            motion: &Motion,
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
        (self.command)(motion, buffer, register_handler, mode);
    }
}

pub fn delete(
    motion: &Motion,
    buffer: &mut Buffer,
    register_handler: &mut RegisterHandler,
    mode: &mut Mode,
) {
}

pub fn yank(
    motion: &Motion,
    buffer: &mut Buffer,
    register_handler: &mut RegisterHandler,
    mode: &mut Mode,
) {
}
