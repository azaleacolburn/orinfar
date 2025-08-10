#![feature(panic_update_hook)]

mod buffer;
mod cli;
mod commands;
mod panic_hook;
mod register;
use std::{
    collections::HashMap,
    io::{stdout, Write},
    path::PathBuf,
};

use clap::Parser;
use commands::Command as Cmd;

use anyhow::Result;
use crossterm::{
    cursor::{DisableBlinking, MoveDown, MoveTo, MoveToColumn},
    event::{read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType, ScrollUp, SetSize},
};

use crate::{
    buffer::Buffer,
    cli::Cli,
    commands::{
        a_cmd, b_cmd, crash, dd_cmd, dollar_cmd, double_quote_cmd, dw_cmd, e_cmd, i_cmd, o_cmd,
        p_cmd, underscore_cmd, w_cmd, x_cmd, O_cmd,
    },
    register::RegisterHandler,
};

pub fn flush_buffer() {}

#[derive(Clone, Debug)]
struct Cursor {
    row: usize,
    col: usize,
}

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

    let commands = vec![
        // Mode Shifting
        Cmd::leaf('i', i_cmd),
        Cmd::leaf('a', a_cmd),
        Cmd::leaf('o', o_cmd),
        Cmd::leaf('O', O_cmd),
        // Movement
        Cmd::leaf('w', w_cmd),
        Cmd::leaf('b', b_cmd),
        Cmd::leaf('e', e_cmd),
        Cmd::leaf('$', dollar_cmd),
        Cmd::leaf('_', underscore_cmd),
        // Editing
        Cmd::leaf('x', x_cmd),
        Cmd::branch('d', [Cmd::leaf('d', dd_cmd), Cmd::leaf('w', dw_cmd)]),
        Cmd::branch('y', [Cmd::leaf('d', dd_cmd), Cmd::leaf('w', dw_cmd)]),
        // Cmd::branch(':', [Cmd::leaf('w', colon_w_cmd)]),
        // Registers
        Cmd::leaf('p', p_cmd),
        Cmd::leaf('c', crash),
        Cmd::leaf('"', double_quote_cmd),
    ];

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
                    let mut current_commands: &Vec<Cmd> = &commands;
                    let mut depth = 0;

                    while let Some(cmd) = current_commands
                        .iter()
                        .find(|cmd| cmd.character == chained[depth])
                    {
                        if cmd.children.is_empty() {
                            (cmd.callback)(&mut buffer, &mut register_handler, &mut mode);
                            chained = vec![];
                            break;
                        } else {
                            current_commands = &cmd.children;
                            depth += 1;
                            if depth == chained.len() {
                                break;
                            }
                        }
                    }
                    register_handler.reset_current_register();
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
                    if buffer.col() > 1 {
                        buffer.prev_col();
                    }
                }
                (KeyCode::Right, _) => {
                    if buffer.col() + 1 < buffer.buff[buffer.row()].len() {
                        buffer.next_col();
                    }
                }
                (KeyCode::Up, _) => {
                    if buffer.row() > 1 {
                        buffer.prev_row();
                        // The cursor can exist one character beyond the last in the buffer
                        buffer.set_col(usize::min(
                            buffer.col(),
                            buffer.buff[buffer.row()].len() - 1,
                        ))
                    }
                }
                (KeyCode::Down, _) => {
                    println!("{}, {}", buffer.row(), buffer.len());
                    println!("{:?}", buffer.buff);
                    if buffer.row() + 1 < buffer.len() {
                        buffer.next_row();
                        buffer.set_col(usize::min(
                            buffer.col(),
                            buffer.buff[buffer.row()].len() - 1,
                        ))
                    }
                }
                _ => continue,
            };

            execute!(stdout, MoveTo(0, 0), Clear(ClearType::All),)?;
            for row in buffer.buff.iter() {
                execute!(
                    stdout,
                    Print(row.clone().into_iter().collect::<String>()),
                    MoveDown(1),
                    MoveToColumn(0),
                )?;
            }
            execute!(stdout, MoveTo(buffer.col() as u16, buffer.row() as u16))?;
            stdout.flush()?;
        }
    }

    cleanup()?;

    Ok(())
}
