use crate::{buffer::Buffer, register::RegisterHandler, Cursor, Mode};

pub struct Operator {
    name: Vec<char>,
    command:
        fn(start: Cursor, buffer: &mut Buffer, registers: &mut RegisterHandler, mode: &mut Mode),
}

impl Operator {
    pub fn new(
        name: &str,
        command: fn(
            start: Cursor,
            buffer: &mut Buffer,
            registers: &mut RegisterHandler,
            mode: &mut Mode,
        ),
    ) -> Self {
        Self {
            name: name.chars().collect(),
            command,
        }
    }
}

pub fn delete(
    start: Cursor,
    buffer: &mut Buffer,
    registers: &mut RegisterHandler,
    mode: &mut Mode,
) {
}

pub struct Operation<'a> {
    operator: &'a Operator,
    start: Cursor,
}

impl<'a> Operation<'a> {
    pub fn execute(self, buffer: &mut Buffer, registers: &mut RegisterHandler, mode: &mut Mode) {
        (self.operator.command)(self.start, buffer, registers, mode);
    }
}
