#![feature(panic_update_hook)]

// Needs to be defined first
#[macro_use]
mod utility;

mod buffer;
mod buffer_char;
mod buffer_line;
mod commands;
mod io;
mod motion;
mod operator;
mod panic_hook;
mod register;

use crate::{
    buffer::Buffer,
    commands::{append, cut, insert, insert_new_line, insert_new_line_above, paste, replace},
    motion::{back, beginning_of_line, end_of_line, end_of_word, find, word, Motion},
    operator::{change, delete, yank, Operator},
    register::RegisterHandler,
};
use anyhow::{bail, Result};
use commands::Command as Cmd;
use crossterm::{
    cursor::{MoveTo, MoveToRow, SetCursorStyle},
    event::{read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ropey::Rope;
use std::{
    fs::OpenOptions,
    io::{stdout, Write},
};

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
    execute!(stdout(), ResetColor, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}

fn log(contents: impl ToString) {
    let mut file = OpenOptions::new()
        .append(true)
        .open("log.txt")
        .expect("Unable to open file");

    // Append data to the file
    file.write_all(format!("{}\n", contents.to_string()).as_bytes())
        .expect("Unable to append data");
}

fn main() -> Result<()> {
    panic_hook::add_panic_hook(&cleanup);

    std::fs::File::create("log.txt")?;

    let mut stdout = stdout();
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

    let (cols, rows) = size()?;
    execute!(
        stdout,
        EnterAlternateScreen,
        Clear(ClearType::All),
        MoveToRow(0),
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
        if buffer.rope.len_chars() == 0 {
            buffer.rope = Rope::from(" ");
        }
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
                    // NOTE I had some stuff here earlier but I'm not sure what I was doing D:
                }

                (KeyCode::Char(c), Mode::Normal) => {
                    // TODO Remove this len_chars thing because pasting
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
                            log("entire line");
                            operation.entire_line(&mut buffer, &mut register_handler, &mut mode);
                            chained.clear();
                            next_operation = None;
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
                    if buffer.get_col() != 0 {
                        buffer.cursor -= 1;
                    }
                    execute!(stdout, SetCursorStyle::SteadyBlock)?;
                    count = 1;
                }
                (KeyCode::Backspace, Mode::Insert) => {
                    let row = buffer.get_row();
                    let col = buffer.get_col();
                    if col > 0 {
                        buffer.rope.remove(col - 1..col);
                        buffer.cursor -= 1;
                    } else if row != 0 {
                        // NOTE We need to create a new rope to sever this slice
                        // from the buffer before appending it again
                        let old_line = Rope::from(buffer.get_curr_line());
                        let old_line_len = old_line.len_chars();
                        buffer.push_slice(old_line.slice(..));
                        buffer.remove_curr_line();
                        buffer.prev_line();
                        buffer.set_col(buffer.get_curr_line().len_chars() - old_line_len);
                    }
                }
                (KeyCode::Char(c), Mode::Insert) => {
                    buffer.insert_char(c);
                    buffer.next_char();
                }
                (KeyCode::Tab, Mode::Insert) => {
                    // NOTE
                    // Iterates two separate times because we want the insertation batched and
                    // the traversal to happen after
                    buffer.insert_char_n_times(' ', 4);
                    (0..4).into_iter().for_each(|_| {
                        buffer.next_char();
                    });
                }
                (KeyCode::Enter, Mode::Insert) => {
                    buffer.insert_char('\n');
                    buffer.cursor += 1;
                }

                (KeyCode::Left, _) => {
                    buffer.prev_char();
                }
                (KeyCode::Right, _) => {
                    buffer.next_char();
                }
                (KeyCode::Up, _) => {
                    if buffer.get_row() > 0 {
                        buffer.prev_line();
                        // panic!("here");

                        let len = buffer.get_curr_line().len_chars();

                        log("up");
                        let col = if len > 0 {
                            usize::min(buffer.get_col() + 1, len - 1) // TODO might be not +1
                        } else {
                            0
                        };
                        buffer.set_col(col)
                    }
                }
                (KeyCode::Down, _) => {
                    log("down");
                    if !buffer.is_last_row() {
                        buffer.next_line();
                        buffer.end_of_line();
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
