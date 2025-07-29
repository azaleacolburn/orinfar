use crate::{
    register::{RegisterHandler, Registers},
    Cursor, Mode,
};
use crossterm::{
    cursor::EnableBlinking,
    event::{read, Event, KeyCode},
    execute,
};
use std::io::{stdout, BufRead};

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
    pub callback: fn(
        buffer: &mut Vec<Vec<char>>,
        cursor: &mut Cursor,
        register_handler: &mut RegisterHandler,
        mode: &mut Mode,
    ) -> (),
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
            buffer: &mut Vec<Vec<char>>,
            cursor: &mut Cursor,
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
pub fn wait(
    _buffer: &mut Vec<Vec<char>>,
    _cursor: &mut Cursor,
    _register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
) {
}
pub fn i_cmd(
    _buffer: &mut Vec<Vec<char>>,
    _cursor: &mut Cursor,
    _register_handler: &mut RegisterHandler,
    mode: &mut Mode,
) {
    *mode = Mode::Insert;
    execute!(stdout(), EnableBlinking).unwrap();
}
pub fn a_cmd(
    buffer: &mut Vec<Vec<char>>,
    cursor: &mut Cursor,
    _register_handler: &mut RegisterHandler,
    mode: &mut Mode,
) {
    *mode = Mode::Insert;
    if cursor.col < buffer[cursor.row].len() {
        cursor.col += 1;
    }
    execute!(stdout(), EnableBlinking).unwrap();
}

pub fn w_cmd(
    buffer: &mut Vec<Vec<char>>,
    cursor: &mut Cursor,
    _register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
) {
    if buffer[cursor.row].len() == 0 {
        return;
    }
    let mut c = buffer[cursor.row][cursor.col]; // = unwrap_or_return!(get_next_char(buffer, cursor));

    if !c.is_alphanumeric() {
        while !c.is_alphanumeric() {
            if cursor.col < buffer[cursor.row].len() {
                cursor.col += 1;
            } else if cursor.row < buffer.len() {
                cursor.row += 1;
                cursor.col = 0;
            } else {
                break;
            }
            c = buffer[cursor.row][cursor.col];
        }
    } else {
        while c.is_alphanumeric() {
            if cursor.col + 1 != buffer[cursor.row].len() {
                cursor.col += 1;
            } else if cursor.row + 1 != buffer.len() {
                cursor.row += 1;
                cursor.col = 0;
            } else {
                break;
            }
            c = buffer[cursor.row][cursor.col];
        }
        while c.is_whitespace() {
            if cursor.col + 1 != buffer[cursor.row].len() {
                cursor.col += 1;
            } else if cursor.row + 1 != buffer.len() {
                cursor.row += 1;
                cursor.col = 0;
            } else {
                break;
            }
            c = buffer[cursor.row][cursor.col];
        }
    }
}

pub fn b_cmd(
    buffer: &mut Vec<Vec<char>>,
    cursor: &mut Cursor,
    _register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
) {
    if buffer[cursor.row].len() == 0 {
        return;
    }
    let mut c = buffer[cursor.row][cursor.col];

    if !c.is_alphanumeric() {
        while !c.is_alphanumeric() {
            if cursor.col > 0 {
                cursor.col -= 1;
            } else if cursor.row > 0 {
                cursor.row -= 1;
                cursor.col = buffer[cursor.row].len() - 1;
            } else {
                break;
            }
            c = buffer[cursor.row][cursor.col];
        }
    } else {
        while c.is_alphanumeric() {
            if cursor.col > 0 {
                cursor.col -= 1;
            } else if cursor.row > 0 {
                cursor.row -= 1;
                cursor.col = buffer[cursor.row].len() - 1;
            } else {
                break;
            }
            c = buffer[cursor.row][cursor.col];
        }
        while c.is_whitespace() {
            if cursor.col > 0 {
                cursor.col -= 1;
            } else if cursor.row > 0 {
                cursor.row -= 1;
                cursor.col = buffer[cursor.row].len() - 1;
            } else {
                break;
            }
            c = buffer[cursor.row][cursor.col];
        }
    }
}

fn get_next_char(buffer: &Vec<Vec<char>>, cursor: &Cursor) -> Option<char> {
    if cursor.col + 1 < buffer[cursor.row].len() {
        Some(buffer[cursor.row][cursor.col + 1])
    } else if cursor.row + 1 < buffer.len() {
        Some(buffer[cursor.row + 1][0])
    } else {
        None
    }
}

pub fn e_cmd(
    buffer: &mut Vec<Vec<char>>,
    cursor: &mut Cursor,
    _register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
) {
    let mut next_char = unwrap_or_return!(get_next_char(buffer, cursor)); // buffer[cursor.row][cursor.col]; // = unwrap_or_return!(get_next_char(buffer, cursor));

    if !next_char.is_alphanumeric() {
        while !next_char.is_alphanumeric() {
            if cursor.col < buffer[cursor.row].len() {
                cursor.col += 1;
            } else if cursor.row < buffer.len() {
                cursor.row += 1;
                cursor.col = 0;
            } else {
                break;
            }
            next_char = unwrap_or_return!(get_next_char(buffer, cursor));
        }
        while next_char.is_alphanumeric() {
            if cursor.col + 1 != buffer[cursor.row].len() {
                cursor.col += 1;
            } else if cursor.row + 1 != buffer.len() {
                cursor.row += 1;
                cursor.col = 0;
            } else {
                break;
            }
            next_char = unwrap_or_return!(get_next_char(buffer, cursor));
        }
    } else {
        while next_char.is_alphanumeric() {
            if cursor.col + 1 != buffer[cursor.row].len() {
                cursor.col += 1;
            } else if cursor.row + 1 != buffer.len() {
                cursor.row += 1;
                cursor.col = 0;
            } else {
                break;
            }
            next_char = unwrap_or_return!(get_next_char(buffer, cursor));
        }
    }
}

pub fn dollar_cmd(
    buffer: &mut Vec<Vec<char>>,
    cursor: &mut Cursor,
    _register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
) {
    cursor.col = buffer[cursor.row].len() - 1
}

pub fn underscore_cmd(
    buffer: &mut Vec<Vec<char>>,
    cursor: &mut Cursor,
    _register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
) {
    let first = buffer[cursor.row]
        .iter()
        .position(|c| !c.is_whitespace())
        .unwrap_or(cursor.col);
    cursor.col = first
}

pub fn x_cmd(
    buffer: &mut Vec<Vec<char>>,
    cursor: &mut Cursor,
    _register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
) {
    if buffer[cursor.row].len() > cursor.col {
        buffer[cursor.row].remove(cursor.col);
    }
}

pub fn o_cmd(
    buffer: &mut Vec<Vec<char>>,
    cursor: &mut Cursor,
    _register_handler: &mut RegisterHandler,
    mode: &mut Mode,
) {
    cursor.row += 1;
    buffer.insert(cursor.row, vec![]);
    cursor.col = 0;
    *mode = Mode::Insert;
}

#[allow(non_snake_case)]
pub fn O_cmd(
    buffer: &mut Vec<Vec<char>>,
    cursor: &mut Cursor,
    _register_handler: &mut RegisterHandler,
    mode: &mut Mode,
) {
    if cursor.row > 0 {
        buffer.insert(cursor.row, vec![]);
        cursor.col = 0;
        *mode = Mode::Insert;
    }
}

pub fn dd_cmd(
    buffer: &mut Vec<Vec<char>>,
    cursor: &mut Cursor,
    _register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
) {
    if buffer.len() > 1 {
        buffer.remove(cursor.row);
        if cursor.row == buffer.len() {
            cursor.row -= 1;
        }
    } else {
        buffer[0] = vec![];
        cursor.col = 0;
    }
}

pub fn dw_cmd(
    buffer: &mut Vec<Vec<char>>,
    cursor: &mut Cursor,
    _register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
) {
    let mut next_char = if cursor.col + 1 != buffer[cursor.row].len() {
        buffer[cursor.row][cursor.col + 1]
    } else if cursor.row + 1 != buffer.len() {
        buffer[cursor.row + 1][0]
    } else {
        return;
    };
    while next_char.is_alphanumeric() {
        if buffer[cursor.row].len() == 0 {
            buffer[cursor.row].remove(cursor.col);
        } else if cursor.col + 1 != buffer[cursor.row].len() {
            buffer[cursor.row].remove(cursor.col);
        } else {
            break;
        }
        next_char = buffer[cursor.row][cursor.col];
    }
    if cursor.col != buffer[cursor.row].len() {
        buffer[cursor.row].remove(cursor.col);
    }
}

pub fn double_quote_cmd(
    _buffer: &mut Vec<Vec<char>>,
    _cursor: &mut Cursor,
    register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
) {
    if let Event::Key(event) = read().unwrap() {
        register_handler.init_reg(event.code, "");
        register_handler.current_register = event.code.to_string();
    }
}
