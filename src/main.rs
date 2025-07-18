mod commands;
use std::{
    io::{stdout, Write},
    process::Command,
};

use commands::Command as Cmd;

use crossterm::{
    cursor::{
        self, position, DisableBlinking, EnableBlinking, MoveDown, MoveLeft, MoveRight, MoveTo,
        MoveToColumn, MoveUp, RestorePosition, SavePosition,
    },
    event::{read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, ScrollDown, ScrollUp, SetSize,
    },
    ExecutableCommand,
};

use crate::commands::{
    b_cmd, dd_cmd, dollar_cmd, dw_cmd, o_cmd, underscore_cmd, w_cmd, x_cmd, O_cmd,
};

#[derive(Clone, Debug)]
struct Cursor {
    row: usize,
    col: usize,
}

#[derive(Clone, Debug)]
enum Mode {
    Normal,
    Insert,
    Visual,
}

fn main() -> std::io::Result<()> {
    let (cols, rows) = size()?;
    let (cursor_cols, cursor_rows) = position()?;
    execute!(
        stdout(),
        SetSize(cols, rows),
        Clear(ClearType::All),
        ScrollUp(rows),
        SetForegroundColor(Color::Blue),
        // SetBackgroundColor(Color::DarkGrey),
    )?;

    // Fill entire screen with spaces with the background color
    for row in 0..rows {
        execute!(stdout(), MoveTo(0, row), Print(" ".repeat(cols as usize)))?;
    }
    execute!(stdout(), MoveTo(0, 0))?;

    enable_raw_mode()?;
    let mut stdout = stdout();

    let mut buffer: Vec<Vec<char>> = vec![vec![]];
    let mut cursor = Cursor { row: 0, col: 0 };

    let commands = vec![
        Cmd::leaf('w', w_cmd),
        Cmd::leaf('b', b_cmd),
        Cmd::leaf('$', dollar_cmd),
        Cmd::leaf('_', underscore_cmd),
        Cmd::leaf('x', x_cmd),
        Cmd::leaf('o', o_cmd),
        Cmd::leaf('O', O_cmd),
        Cmd::branch('d', [Cmd::leaf('d', dd_cmd), Cmd::leaf('w', dw_cmd)]),
    ];

    let mut mode = Mode::Normal;

    let mut count: u16 = 1;
    let mut chained: Vec<char> = vec![];

    loop {
        if let Event::Key(event) = read()? {
            match (event.code, mode.clone()) {
                (KeyCode::Char(c), Mode::Normal) if c == 'i' => {
                    mode = Mode::Insert;
                    execute!(stdout, EnableBlinking)?;
                }
                (KeyCode::Char(c), Mode::Normal) if c == 'q' => break,
                (KeyCode::Char(c), Mode::Normal) if c.is_numeric() => {
                    let c = c.to_digit(10).unwrap() as u16;
                    if count == 1 {
                        count = 0;
                    }
                    count *= 10;
                    count += c;
                }
                // (KeyCode::Char(c), Mode::Normal) => {
                //     chained.push(c);
                //     let mut matched_list = commands.iter().filter(|cmd| chained == cmd.chain);
                //     if let Some(matched) = matched_list.next() {
                //         assert!(matched_list.next().is_none());
                //         for _ in 0..count {
                //             (matched.callback)(&mut buffer, &mut cursor, &mut mode);
                //         }
                //         count = 1;
                //         chained = vec![];
                //     }
                // }
                (KeyCode::Char(c), Mode::Normal) => {
                    chained.push(c);

                    let mut array: &Vec<Cmd> = &commands;
                    let mut depth = 0;
                    loop {
                        match array.iter().find(|cmd| cmd.character == chained[depth]) {
                            Some(cmd) => {
                                if cmd.children.len() == 0 {
                                    (cmd.callback)(&mut buffer, &mut cursor, &mut mode);
                                    chained = vec![];
                                    break;
                                } else {
                                    array = &cmd.children;
                                    depth += 1;
                                    if depth == chained.len() {
                                        break;
                                    }
                                    continue;
                                }
                            }
                            None => break,
                        }
                    }
                }
                (KeyCode::Esc, Mode::Insert) => {
                    mode = Mode::Normal;
                    execute!(stdout, DisableBlinking)?;
                    count = 1;
                }
                (KeyCode::Backspace, Mode::Insert) => {
                    if cursor.col > 0 {
                        buffer[cursor.row].remove(cursor.col - 1);
                        cursor.col -= 1;
                    } else if cursor.row != 0 {
                        let mut old_line = buffer[cursor.row].clone();
                        buffer[cursor.row - 1].append(&mut old_line);
                        buffer.remove(cursor.row);
                        cursor.row -= 1;
                        cursor.col = buffer[cursor.row].len()
                    }
                }
                (KeyCode::Char(c), Mode::Insert) => {
                    buffer[cursor.row].insert(cursor.col, c);
                    cursor.col += 1;
                }
                (KeyCode::Enter, Mode::Insert) => {
                    let end = match buffer[cursor.row].len() > 0 {
                        true => buffer[cursor.row].split_off(cursor.col),
                        false => vec![],
                    };

                    buffer.push(end);
                    cursor.col = 0;
                    cursor.row += 1;
                }

                (KeyCode::Left, _) => {
                    if cursor.col > 0 {
                        cursor.col -= 1;
                    }
                }
                (KeyCode::Right, _) => {
                    if cursor.col < buffer[cursor.row].len() {
                        cursor.col += 1;
                    }
                }
                (KeyCode::Up, _) => {
                    if cursor.row > 0 {
                        cursor.row -= 1;
                        // The cursor can exist one character beyond the last in the buffer
                        cursor.col = usize::min(cursor.col, buffer[cursor.row].len())
                    }
                }
                (KeyCode::Down, _) => {
                    if cursor.row < buffer.len() - 1 {
                        cursor.row += 1;
                        cursor.col = usize::min(cursor.col, buffer[cursor.row].len())
                    }
                }
                _ => continue,
            };

            execute!(stdout, MoveTo(0, 0), Clear(ClearType::All),)?;
            for row in buffer.iter() {
                execute!(
                    stdout,
                    Print(row.clone().into_iter().collect::<String>()),
                    MoveDown(1),
                    MoveToColumn(0),
                )?;
            }
            execute!(stdout, MoveTo(cursor.col as u16, cursor.row as u16))?;
            stdout.flush()?;
        }
    }

    execute!(stdout, SetSize(cols, rows), ResetColor)?;
    disable_raw_mode()?;

    Ok(())
}
