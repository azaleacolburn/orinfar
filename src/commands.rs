use crate::{buffer::Buffer, register::RegisterHandler, Mode};
use crossterm::{
    cursor::SetCursorStyle,
    event::{read, Event, KeyCode},
    execute,
};
use std::io::stdout;

pub struct Command<'a> {
    pub name: &'a [char],
    command: fn(buffer: &mut Buffer, register_handler: &mut RegisterHandler, mode: &mut Mode),
}

impl<'a> Command<'a> {
    pub fn new(
        name: &'a [char],
        command: fn(buffer: &mut Buffer, register_handler: &mut RegisterHandler, mode: &mut Mode),
    ) -> Self {
        Command { name, command }
    }

    pub fn inconslusive(
        name: &'a [char],
        command: fn(buffer: &mut Buffer, register_handler: &mut RegisterHandler, mode: &mut Mode),
    ) -> Self {
        Command { name, command }
    }

    pub fn execute(
        &self,
        buffer: &mut Buffer,
        register_handler: &mut RegisterHandler,
        mode: &mut Mode,
    ) {
        (self.command)(buffer, register_handler, mode)
    }
}

pub fn noop(_buffer: &mut Buffer, _register_handler: &mut RegisterHandler, _mode: &mut Mode) {}

pub fn insert(_buffer: &mut Buffer, _register_handler: &mut RegisterHandler, mode: &mut Mode) {
    mode.insert();
}

pub fn append(buffer: &mut Buffer, _register_handler: &mut RegisterHandler, mode: &mut Mode) {
    if buffer.cursor != buffer.rope.len_chars() {
        buffer.cursor += 1;
        mode.insert();
    }
}

pub fn cut(buffer: &mut Buffer, _register_handler: &mut RegisterHandler, _mode: &mut Mode) {
    if buffer.rope.get_char(buffer.cursor) == Some('\n') {
        return;
    }
    buffer.rope.remove(buffer.cursor..buffer.cursor);
}

pub fn insert_new_line(
    buffer: &mut Buffer,
    _register_handler: &mut RegisterHandler,
    mode: &mut Mode,
) {
    buffer.rope.insert_char(buffer.cursor, '\n');
    buffer.cursor += 1;
    mode.insert();
}

pub fn insert_new_line_above(
    buffer: &mut Buffer,
    _register_handler: &mut RegisterHandler,
    mode: &mut Mode,
) {
    let first = buffer
        .rope
        .line_to_char(buffer.rope.char_to_line(buffer.cursor))
        - 1;
    buffer.rope.insert_char(first, '\n');
    buffer.cursor = first;
    mode.insert();
}

pub fn paste(buffer: &mut Buffer, register_handler: &mut RegisterHandler, _mode: &mut Mode) {
    let contents = &register_handler.get_reg();
    buffer.rope.insert(buffer.cursor, contents);
}

pub fn crash(_buffer: &mut Buffer, _register_handler: &mut RegisterHandler, _mode: &mut Mode) {
    panic!("Intentionally Crashed")
}

pub fn double_quote_cmd(
    _buffer: &mut Buffer,
    register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
) {
    if let Event::Key(event) = read().unwrap() {
        register_handler.init_reg(event.code, "");
        register_handler.current_register = event.code.to_string();
    }
}

pub fn replace(buffer: &mut Buffer, _register_handler: &mut RegisterHandler, _mode: &mut Mode) {
    execute!(stdout(), SetCursorStyle::SteadyUnderScore).unwrap();
    if let Event::Key(event) = read().unwrap() {
        if let KeyCode::Char(c) = event.code {
            buffer.replace_curr_char(c);
        }
    }
    execute!(stdout(), SetCursorStyle::SteadyBlock).unwrap();
}
