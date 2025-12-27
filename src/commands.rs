use crate::{Mode, buffer::Buffer, log, register::RegisterHandler};
use crossterm::{
    cursor::SetCursorStyle,
    event::{Event, KeyCode, read},
    execute,
};
use std::io::stdout;

pub struct Command<'a> {
    pub name: &'a str,
    command: fn(buffer: &mut Buffer, register_handler: &mut RegisterHandler, mode: &mut Mode),
}

impl<'a> Command<'a> {
    pub fn new(
        name: &'a str,
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

pub fn insert(buffer: &mut Buffer, _register_handler: &mut RegisterHandler, mode: &mut Mode) {
    mode.insert();
}

pub fn append(buffer: &mut Buffer, _register_handler: &mut RegisterHandler, mode: &mut Mode) {
    if buffer.cursor != buffer.get_end_of_line() || buffer.cursor + 1 == buffer.rope.len_chars() {
        buffer.cursor += 1;
    }

    mode.insert();
}

pub fn cut(buffer: &mut Buffer, register_handler: &mut RegisterHandler, _mode: &mut Mode) {
    if let Some(c) = buffer.rope.get_char(buffer.cursor) {
        if c == '\n' {
            return;
        }
        register_handler.set_reg(c.to_string());

        buffer.rope.remove(buffer.cursor..=buffer.cursor);
        if buffer.cursor != 0 && buffer.is_last_col() {
            buffer.cursor -= 1;
        }
        buffer.update_list_use_current_line();
    }
}
pub fn insert_new_line(
    buffer: &mut Buffer,
    register_handler: &mut RegisterHandler,
    mode: &mut Mode,
) {
    buffer.end_of_line();
    append(buffer, register_handler, mode);
    buffer.insert_char('\n');
    buffer.cursor += 1;
}

pub fn insert_new_line_above(
    buffer: &mut Buffer,
    _register_handler: &mut RegisterHandler,
    mode: &mut Mode,
) {
    let line_idx = buffer.rope.char_to_line(buffer.cursor);
    let first = buffer.rope.line_to_char(line_idx);
    buffer.update_list_add(line_idx);
    buffer.rope.insert_char(first, '\n');

    buffer.cursor = first;
    mode.insert();
    buffer.has_changed = true;
}

pub fn paste(buffer: &mut Buffer, register_handler: &mut RegisterHandler, _mode: &mut Mode) {
    let contents = &register_handler.get_reg();

    let line_idx = buffer.get_row();
    contents
        .chars()
        .filter(|c| *c == '\n')
        .for_each(|_| buffer.update_list_add(line_idx));

    buffer.rope.insert(buffer.cursor, contents);
    buffer.update_list_use_current_line();
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
    buffer.has_changed = true;
}

pub fn last_row(buffer: &mut Buffer, _register_handler: &mut RegisterHandler, _mode: &mut Mode) {
    let last_row = usize::max(buffer.len(), 1) - 1;
    buffer.set_row(last_row);
}

pub fn first_row(buffer: &mut Buffer, _register_handler: &mut RegisterHandler, _mode: &mut Mode) {
    buffer.set_row(0);
}
