#![feature(panic_update_hook)]

// Needs to be defined first
#[macro_use]
mod utility;

mod buffer;
mod cli;
mod commands;
mod motion;
mod operator;
mod panic_hook;
mod register;

use crate::{
    buffer::{Buffer, Cursor},
    cli::Cli,
    commands::{append, cut, insert, o_cmd, paste, O_cmd},
    motion::{back, beginning_of_line, end_of_line, end_of_word, word, Motion},
    operator::{change, delete, yank, Operator},
    register::RegisterHandler,
};
use anyhow::Result;
use clap::Parser;
use commands::Command as Cmd;
use crossterm::{
    cursor::{DisableBlinking, MoveTo},
    event::{read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType, ScrollUp, SetSize},
};
use std::{collections::HashMap, io::stdout, path::PathBuf};

#[derive(Clone, Debug)]
enum Mode {
    Normal,
    Insert,
    _Visual,
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

    let mut register_handler = RegisterHandler::new();
    let mut _marks: HashMap<char, (usize, usize)> = HashMap::new();
    let mut buffer: Buffer = Buffer::new();

    let commands: &[Cmd] = &[
        Cmd::new("i", insert),
        Cmd::new("p", paste),
        Cmd::new("a", append),
        Cmd::new("o", o_cmd),
        Cmd::new("O", O_cmd),
        Cmd::new("x", cut),
    ];
    let operators: &[Operator] = &[
        Operator::new("d", delete),
        Operator::new("y", yank),
        Operator::new("c", change),
    ];
    let motions: &[Motion] = &[
        Motion::new("w", word),
        Motion::new("b", back),
        Motion::new("e", end_of_word),
        Motion::new("$", end_of_line),
        Motion::new("_", beginning_of_line),
    ];
    let mut next_operation: Option<&Operator> = None;
    let mut pending_motion: Option<&Motion> = None;

    let cli = Cli::parse();
    // TODO This is a bad way of handling things, refactor later
    let path = match cli.file_name {
        Some(path) => {
            let path = PathBuf::from(path);

            if path.is_dir() {
                // TODO netrw
                return Ok(());
            } else if path.is_file() {
                let contents = std::fs::read_to_string(path.clone())?;
                contents
                    .split('\n')
                    .for_each(|line| buffer.push_line(line.chars().collect::<Vec<char>>()));
            }
            Some(path)
        }
        None => None,
    };

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
                                Some(path) => std::fs::write(&path, buffer.to_string())?,
                                None => {
                                    // TODO Cursorline system for error handling
                                }
                            }
                            break;
                        }
                    }
                    let end = buffer.get_line_end();
                    buffer.push_line(end);
                    buffer.set_col(0);
                    buffer.next_row();
                }

                (KeyCode::Char(c), Mode::Normal) => {
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

                    if let Some(motion) = pending_motion {}
                }

                (KeyCode::Esc, Mode::Insert) => {
                    mode = Mode::Normal;
                    if buffer.col() != 0 {
                        buffer.prev_col();
                    }
                    execute!(stdout, DisableBlinking)?;
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
                        buffer.buff[row - 1].append(&mut old_line);
                        buffer.remove_line(row);
                        buffer.prev_row();
                        buffer.set_col(buffer.buff[buffer.row()].len());
                    }
                }
                (KeyCode::Char(c), Mode::Insert) => {
                    buffer.insert_char_at_cursor(c);
                    buffer.next_col();
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
                        // The cursor can exist one character beyond the last in the buffer
                        buffer.set_col(usize::min(
                            buffer.col(),
                            buffer.buff[buffer.row()].len() - 1,
                        ))
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
