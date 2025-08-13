use crate::{buffer::Buffer, register::RegisterHandler, Cursor, Mode};
use crossterm::{
    cursor::EnableBlinking,
    event::{read, Event},
    execute,
};
use std::io::stdout;

macro_rules! unwrap_or_return {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None => return,
        }
    };
}

pub struct Command {
    pub character: char,
    pub callback: fn(buffer: &mut Buffer, register_handler: &mut RegisterHandler, mode: &mut Mode),
    pub children: Vec<Command>,
}

impl Command {
    pub fn branch(character: char, children: impl Into<Vec<Command>>) -> Self {
        Command {
            character,
            callback: wait,
            children: children.into(),
        }
    }
    pub fn leaf(
        character: char,
        callback: fn(
            buffer: &mut Buffer,
            register_handler: &mut RegisterHandler,
            mode: &mut Mode,
        ) -> (),
    ) -> Self {
        Command {
            character,
            callback,
            children: Vec::new(),
        }
    }
}

// Callbacks
//
pub fn wait(_buffer: &mut Buffer, _register_handler: &mut RegisterHandler, _mode: &mut Mode) {}

pub fn i_cmd(_buffer: &mut Buffer, _register_handler: &mut RegisterHandler, mode: &mut Mode) {
    *mode = Mode::Insert;
    execute!(stdout(), EnableBlinking).unwrap();
}

pub fn a_cmd(buffer: &mut Buffer, _register_handler: &mut RegisterHandler, mode: &mut Mode) {
    *mode = Mode::Insert;
    buffer.next_col();
    execute!(stdout(), EnableBlinking).unwrap();
}

macro_rules! next_char {
    ($buffer:ident, $c:ident) => {
        if $buffer.cursor.col + 1 < $buffer.buff[$buffer.cursor.row].len() {
            $buffer.cursor.col += 1;
        } else if $buffer.cursor.row + 1 < $buffer.buff.len() {
            $buffer.cursor.row += 1;
            $buffer.cursor.col = 0;
        } else {
            break;
        }
        $c = $buffer.get_curr_char();
    };
}

macro_rules! prev_char {
    ($buffer:expr, $c:expr) => {
        if $buffer.cursor.col > 0 {
            $buffer.cursor.col -= 1;
        } else if $buffer.cursor.row > 0 {
            $buffer.cursor.row -= 1;
            $buffer.cursor.col = $buffer.buff[$buffer.cursor.row].len() - 1;
        } else {
            break;
        }
        $c = $buffer.get_curr_char();
    };
}

// TODO newlines aren't actually represented, so the w command system doesn't exactly work as
// expected
pub fn w_cmd(buffer: &mut Buffer, _register_handler: &mut RegisterHandler, _mode: &mut Mode) {
    if buffer.is_empty_line() {
        return;
    }
    let mut c = buffer.get_curr_char();

    if !c.is_alphanumeric() {
        while !c.is_alphanumeric() {
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

pub fn b_cmd(buffer: &mut Buffer, _register_handler: &mut RegisterHandler, _mode: &mut Mode) {
    if buffer.buff[buffer.cursor.row].is_empty() {
        return;
    }
    let mut c = buffer.get_curr_char();

    if !c.is_alphanumeric() {
        while !c.is_alphanumeric() {
            prev_char!(buffer, c);
        }
    } else {
        while c.is_alphanumeric() {
            prev_char!(buffer, c);
        }
        while c.is_whitespace() {
            prev_char!(buffer, c);
        }
    }
}

pub fn e_cmd(buffer: &mut Buffer, _register_handler: &mut RegisterHandler, _mode: &mut Mode) {
    let mut next_char = unwrap_or_return!(buffer.get_next_char());

    if !next_char.is_alphanumeric() {
        while !next_char.is_alphanumeric() {
            next_char = unwrap_or_return!(buffer.next_char());
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
            next_char = unwrap_or_return!(buffer.next_char());
        }
    }
}

pub fn dollar_cmd(buffer: &mut Buffer, _register_handler: &mut RegisterHandler, _mode: &mut Mode) {
    buffer.cursor.col = buffer.buff[buffer.cursor.row].len() - 1
}

pub fn underscore_cmd(
    buffer: &mut Buffer,
    _register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
) {
    let first = buffer.buff[buffer.cursor.row]
        .iter()
        .position(|c| !c.is_whitespace())
        .unwrap_or(buffer.cursor.col);
    buffer.cursor.col = first
}

pub fn x_cmd(buffer: &mut Buffer, _register_handler: &mut RegisterHandler, _mode: &mut Mode) {
    if buffer.buff[buffer.cursor.row].len() > buffer.cursor.col {
        buffer.remove_char(buffer.cursor.col);
        if buffer.buff[buffer.cursor.row].len() == buffer.cursor.col && buffer.cursor.col != 0 {
            buffer.cursor.col -= 1;
        }
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

pub fn dw_cmd(buffer: &mut Buffer, _register_handler: &mut RegisterHandler, _mode: &mut Mode) {
    if buffer.buff[buffer.cursor.row].is_empty() {
        return;
    }
    let mut c = buffer.buff[buffer.cursor.row][buffer.cursor.col];

    if !c.is_alphanumeric() {
        while !c.is_alphanumeric() {
            if buffer.cursor.col < buffer.buff[buffer.cursor.row].len() {
                buffer.remove_char(buffer.cursor.col);
                if buffer.buff[buffer.cursor.row].is_empty() {
                    break;
                }
            } else {
                break;
            }
            c = buffer.buff[buffer.cursor.row][buffer.cursor.col];
        }
    } else {
        while c.is_alphanumeric() {
            if buffer.cursor.col < buffer.buff[buffer.cursor.row].len() {
                buffer.remove_char(buffer.cursor.col);
                if buffer.buff[buffer.cursor.row].is_empty() {
                    break;
                }
            } else {
                break;
            }
            c = buffer.buff[buffer.cursor.row][buffer.cursor.col];
        }
        while c.is_whitespace() {
            if buffer.cursor.col < buffer.buff[buffer.cursor.row].len() {
                buffer.remove_char(buffer.cursor.col);
                if buffer.buff[buffer.cursor.row].is_empty() {
                    break;
                }
            } else {
                break;
            }
            c = buffer.buff[buffer.cursor.row][buffer.cursor.col];
        }
    }
}

pub fn p_cmd(buffer: &mut Buffer, register_handler: &mut RegisterHandler, _mode: &mut Mode) {
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
