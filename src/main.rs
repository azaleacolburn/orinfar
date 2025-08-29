#![feature(panic_update_hook)]

// Needs to be defined first
#[macro_use]
mod utility;

mod buffer;
mod commands;
mod io;
mod motion;
mod operator;
mod panic_hook;
mod register;

use crate::{
    buffer::{Buffer, Cursor},
    commands::{append, cut, insert, insert_new_line, insert_new_line_above, paste, replace},
    motion::{back, beginning_of_line, end_of_line, end_of_word, find, word, Motion},
    operator::{change, delete, yank, Operator},
    register::RegisterHandler,
};
use anyhow::{bail, Result};
use commands::Command as Cmd;
use crossterm::{
    cursor::{DisableBlinking, MoveTo, SetCursorStyle},
    event::{read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType, ScrollUp, SetSize},
};
use std::io::stdout;

#[derive(Clone, Debug)]
enum Mode {
    Normal,
    Insert,
    _Visual,
}

impl Mode {
    fn insert(&mut self) {
        *self = Mode::Insert;
        execute!(stdout(), SetCursorStyle::BlinkingBar).unwrap();
    }

    fn normal(&mut self) {
        *self = Mode::Normal;
        execute!(stdout(), SetCursorStyle::SteadyBlock).unwrap();
    }
}

fn cleanup() -> Result<()> {
    execute!(stdout(), ResetColor)?;
    disable_raw_mode()?;

    Ok(())
}

fn main() -> Result<()> {
    panic_hook::add_panic_hook(&cleanup);

    let mut stdout = stdout();
    let (cols, rows) = size()?;
    let _leader = ' ';

    let mut register_handler = RegisterHandler::new();
    let mut buffer: Buffer = Buffer::new();

    let commands: &[Cmd] = &[
        Cmd::new(&['i'], insert),
        Cmd::new(&['p'], paste),
        Cmd::new(&['a'], append),
        Cmd::new(&['o'], insert_new_line),
        Cmd::new(&['O'], insert_new_line_above),
        Cmd::new(&['x'], cut),
        Cmd::new(&['r'], replace),
    ];
    let operators: &[Operator] = &[
        Operator::new(&['d'], delete),
        Operator::new(&['y'], yank),
        Operator::new(&['c'], change),
    ];
    let motions: &[Motion] = &[
        Motion::new(&['w'], word),
        Motion::new(&['b'], back),
        Motion::new(&['e'], end_of_word),
        Motion::new(&['$'], end_of_line),
        Motion::new(&['_'], beginning_of_line),
        Motion::new(&['f'], find),
    ];
    let mut next_operation: Option<&Operator> = None;

    // Used for not putting excluded chars in the chain
    let command_chars = commands.iter().map(|cmd| cmd.name).flatten();
    let operator_chars = operators.iter().map(|cmd| cmd.name).flatten();
    let motion_chars = motions.iter().map(|cmd| cmd.name).flatten();
    let all_normal_chars: Vec<char> = command_chars
        .chain(operator_chars)
        .chain(motion_chars)
        .map(|n| *n)
        .collect();

    execute!(
        stdout,
        SetSize(cols, rows),
        Clear(ClearType::All),
        ScrollUp(rows),
        SetForegroundColor(Color::Blue),
    )?;

    // Fill entire screen with spaces with the background color
    for row in 0..rows {
        execute!(stdout, MoveTo(0, row), Print(" ".repeat(cols as usize)))?;
    }
    execute!(stdout, MoveTo(0, 0))?;
    for row in 0..rows {
        execute!(stdout, MoveTo(0, row), Print(" ".repeat(cols as usize)))?;
    }
    execute!(stdout, MoveTo(0, 0))?;
    enable_raw_mode()?;

    let path = io::load_file(&mut buffer)?;

    let mut mode = Mode::Normal;
    let mut count: u16 = 1;
    let mut chained: Vec<char> = vec![];

    loop {
        if let Event::Key(event) = read()? {
            match (event.code, mode.clone()) {
                (KeyCode::Char('q'), Mode::Normal) => break,
                (KeyCode::Char(c), Mode::Normal) if c.is_numeric() => {
                    let c = c.to_digit(10).unwrap() as u16;
                    if count == 1 {
                        count = 0;
                    }
                    count *= 10;
                    count += c;
                }
                (KeyCode::Char(':'), Mode::Normal) => {
                    if let Event::Key(event) = read()? {
                        if event.code == KeyCode::Char('w') {
                            match path {
                                Some(path) => {
                                    io::write(path, buffer)?;
                                    break;
                                }
                                None => bail!("Cannot write buffer, no file opened."),
                            }
                        }
                    }
                    let end = buffer.get_line_end();
                    buffer.push_line(end);
                    buffer.set_col(0);
                    buffer.next_row();
                }

                (KeyCode::Char(c), Mode::Normal) => {
                    if !all_normal_chars.contains(&c) {
                        continue;
                    };
                    chained.push(c);

                    if let Some(command) = commands.iter().find(|motion| motion.name == chained) {
                        command.execute(&mut buffer, &mut register_handler, &mut mode);
                        chained.clear();
                    } else if let Some(operation) = next_operation {
                        if let Some(motion) = motions.iter().find(|motion| motion.name[0] == c) {
                            operation.execute(
                                motion,
                                &mut buffer,
                                &mut register_handler,
                                &mut mode,
                            );
                            chained.clear();
                            next_operation = None;
                        } else if c == operation.name[0] {
                            operation.entire_line(&mut buffer, &mut register_handler, &mut mode);
                        }
                    } else if chained.len() == 1 {
                        if let Some(motion) = motions.iter().find(|motion| motion.name[0] == c) {
                            motion.apply(&mut buffer);
                            chained.clear();
                        }
                    }

                    if let Some(operator) =
                        operators.iter().find(|operator| operator.name == chained)
                    {
                        next_operation = Some(&operator);
                    }
                }

                (KeyCode::Esc, Mode::Insert) => {
                    mode = Mode::Normal;
                    if buffer.col() != 0 {
                        buffer.prev_col();
                    }
                    execute!(stdout, SetCursorStyle::SteadyBlock)?;
                    count = 1;
                }
                (KeyCode::Backspace, Mode::Insert) => {
                    let row = buffer.row();
                    let col = buffer.col();
                    if buffer.col() > 0 {
                        buffer.remove_char(col - 1);
                        buffer.prev_col();
                    } else if buffer.row() != 0 {
                        let mut old_line = buffer.buff[buffer.row()].clone();
                        let old_line_len = old_line.len();
                        buffer.buff[row - 1].append(&mut old_line);
                        buffer.remove_line(row);
                        buffer.prev_row();
                        buffer.set_col(buffer.buff[row - 1].len() - old_line_len);
                    }
                }
                (KeyCode::Char(c), Mode::Insert) => {
                    buffer.insert_char_at_cursor(c);
                    buffer.next_col();
                }
                (KeyCode::Tab, Mode::Insert) => {
                    // NOTE
                    // Iterates two separate times because we want the insertation batched and
                    // the traversal to happen after
                    buffer.insert_n_char(' ', 4);
                    (0..4).into_iter().for_each(|_| {
                        buffer.next_col();
                    });
                }
                (KeyCode::Enter, Mode::Insert) => {
                    let end = buffer.get_line_end();

                    buffer.push_line(end);
                    buffer.set_col(0);
                    buffer.next_row();
                }

                (KeyCode::Left, _) => {
                    buffer.prev_col();
                }
                (KeyCode::Right, _) => {
                    buffer.next_col();
                }
                (KeyCode::Up, _) => {
                    if buffer.row() > 0 {
                        buffer.prev_row();

                        let len = buffer.get_curr_line().len();
                        let col = if len > 0 {
                            usize::min(buffer.col() + 1, len - 1)
                        } else {
                            0
                        };
                        buffer.set_col(col)
                    }
                }
                (KeyCode::Down, _) => {
                    if buffer.row() + 1 < buffer.len() {
                        buffer.next_row();
                        buffer.end_of_row();
                    }
                }
                _ => continue,
            };

            buffer.flush()?;
        }
    }

    cleanup()
}

pub fn on_next_input_buffer_only(
    buffer: &mut Buffer,
    closure: fn(KeyCode, &mut Buffer),
) -> Result<()> {
    loop {
        if let Event::Key(event) = read()? {
            closure(event.code, buffer);
            break;
        }
    }

    Ok(())
}

pub fn on_next_input(
    buffer: &mut Buffer,
    mode: &mut Mode,
    register_handler: &mut RegisterHandler,
    count: &mut usize,
    chained: &mut Vec<char>,

    closure: fn(KeyCode, &mut Buffer, &mut Mode, &mut RegisterHandler, &mut usize, &mut Vec<char>),
) -> Result<()> {
    loop {
        if let Event::Key(event) = read()? {
            closure(event.code, buffer, mode, register_handler, count, chained);
            break;
        }
    }

    Ok(())
}
