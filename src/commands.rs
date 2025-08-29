use crate::{buffer::Buffer, register::RegisterHandler, Mode};
use crossterm::{
    cursor::EnableBlinking,
    event::{read, Event},
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
    *mode = Mode::Insert;
    execute!(stdout(), EnableBlinking).unwrap();
}

pub fn append(buffer: &mut Buffer, _register_handler: &mut RegisterHandler, mode: &mut Mode) {
    *mode = Mode::Insert;
    buffer.next_col();
    execute!(stdout(), EnableBlinking).unwrap();
}

pub fn cut(buffer: &mut Buffer, _register_handler: &mut RegisterHandler, _mode: &mut Mode) {
    if buffer.buff[buffer.cursor.row].len() <= buffer.cursor.col {
        return;
    }

    buffer.remove_char(buffer.cursor.col);
    if buffer.buff[buffer.cursor.row].len() == buffer.cursor.col && buffer.cursor.col != 0 {
        buffer.cursor.col -= 1;
    }
}

pub fn o_cmd(buffer: &mut Buffer, _register_handler: &mut RegisterHandler, mode: &mut Mode) {
    buffer.cursor.row += 1;
    buffer.insert_line(buffer.cursor.row, vec![]);
    buffer.cursor.col = 0;
    *mode = Mode::Insert;
}

#[allow(non_snake_case)]
pub fn O_cmd(buffer: &mut Buffer, _register_handler: &mut RegisterHandler, mode: &mut Mode) {
    if buffer.cursor.row > 0 {
        buffer.insert_line(buffer.cursor.row, vec![]);
        buffer.cursor.col = 0;
        *mode = Mode::Insert;
    }
}

pub fn dd_cmd(buffer: &mut Buffer, register_handler: &mut RegisterHandler, _mode: &mut Mode) {
    if buffer.len() > 1 {
        let line = buffer.remove_line(buffer.cursor.row);
        register_handler.set_reg(line);
        if buffer.cursor.row == buffer.len() {
            buffer.cursor.row -= 1;
        }
    } else {
        register_handler.set_reg(buffer.buff[0].clone());
        buffer.buff[0] = vec![];
        buffer.cursor.col = 0;
    }
}
pub fn paste(buffer: &mut Buffer, register_handler: &mut RegisterHandler, _mode: &mut Mode) {
    let mut i = buffer.cursor.col;
    register_handler.get_reg().iter().for_each(|c| {
        buffer.buff[buffer.cursor.row].insert(i, *c);
        i += 1;
    });
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
        register_handler.init_reg(event.code, Vec::default());
        register_handler.current_register = event.code.to_string();
    }
}

// pub fn colon_w_cmd(
//
//     buffer: &mut Vec<Vec<char>>,
//     _buffer.cursor: &mut Cursor,
//     register_handler: &mut RegisterHandler,
//     _mode: &mut Mode,
// ) {
//
// }
//
