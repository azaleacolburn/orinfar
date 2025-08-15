use crate::{buffer::Buffer, motion::Motion, register::RegisterHandler, Cursor, Mode};

pub struct Operator {
    pub name: Vec<char>,
    command:
        fn(motion: &Motion, buffer: &mut Buffer, registers: &mut RegisterHandler, mode: &mut Mode),
}

impl Operator {
    pub fn new(
        name: &str,
        command: fn(
            motion: &Motion,
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

    pub fn execute(self) {
        (self.command)
    }
}

pub fn delete(
    motion: &Motion,
    buffer: &mut Buffer,
    registers: &mut RegisterHandler,
    mode: &mut Mode,
) {
}

pub fn yank(
    motion: &Motion,
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
