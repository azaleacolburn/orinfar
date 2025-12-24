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
mod status_bar;
mod view_box;

use crate::{
    buffer::Buffer,
    commands::{append, cut, insert, insert_new_line, insert_new_line_above, paste, replace},
    io::Cli,
    motion::{back, beginning_of_line, end_of_line, end_of_word, find, word, Motion},
    operator::{change, change_until_before, delete, yank, Operator},
    register::RegisterHandler,
    status_bar::StatusBar,
    view_box::ViewBox,
};
use anyhow::Result;
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
use std::{
    fs::OpenOptions,
    io::{stdout, Write},
    path::PathBuf,
    u16,
};

#[derive(Clone, Debug)]
enum Mode {
    Normal,
    Insert,
    Command,
    Visual,
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
    disable_raw_mode()?;
    execute!(
        stdout(),
        ResetColor,
        Clear(ClearType::All),
        SetCursorStyle::SteadyBlock,
        LeaveAlternateScreen
    )?;

    Ok(())
}

fn setup(rows: u16, cols: u16) -> Result<()> {
    execute!(
        stdout(),
        EnterAlternateScreen,
        Clear(ClearType::All),
        MoveToRow(0),
        SetForegroundColor(Color::Blue),
    )?;

    // Fill entire screen with spaces with the background color
    for row in 0..rows {
        execute!(stdout(), MoveTo(0, row), Print(" ".repeat(cols as usize)))?;
    }
    execute!(stdout(), MoveTo(0, 0))?;
    for row in 0..rows {
        execute!(stdout(), MoveTo(0, row), Print(" ".repeat(cols as usize)))?;
    }
    execute!(stdout(), MoveTo(0, 0))?;
    enable_raw_mode()?;

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
    let mut status_bar: StatusBar = StatusBar::new();

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
        Operator::new(&['t'], change_until_before),
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
    let mut view_box: ViewBox = ViewBox::new(cols, rows);
    setup(rows, cols);

    let mut mode = Mode::Normal;
    let mut count: u16 = 1;
    let mut chained: Vec<char> = vec![];

    let (_cli, mut path) = Cli::parse_path()?;
    io::load_file(&path, &mut buffer)?;
    view_box.flush(&mut buffer, &status_bar, &mode, &path, false)?;

    'main: loop {
        buffer.has_changed = false;
        if let Event::Key(event) = read()? {
            match (event.code, mode.clone()) {
                (KeyCode::Char(c), Mode::Normal) if c.is_numeric() => {
                    let c = c.to_digit(10).unwrap() as u16;
                    if count == 1 {
                        count = 0;
                    }
                    count *= 10;
                    count += c;
                }
                (KeyCode::Char(':'), Mode::Normal) => {
                    mode = Mode::Command;
                    status_bar.push(':');
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
                    if buffer.cursor == 0 {
                        continue;
                    }
                    buffer.cursor -= 1;
                    buffer.delete_curr_char();
                    buffer.has_changed = true;
                }
                (KeyCode::Char(c), Mode::Insert) => {
                    buffer.insert_char(c);
                    // if buffer.rope.len_chars() > 1 {
                    buffer.cursor += 1;
                    buffer.has_changed = true;
                    // }
                    // buffer.next_char();
                    // panic!(
                    //     "buffer: {:?} {}",
                    //     buffer.rope.bytes().collect::<Vec<u8>>(),
                    //     buffer.cursor
                    // );
                }
                (KeyCode::Tab, Mode::Insert) => {
                    // NOTE
                    // Iterates two separate times because we want the insertation batched and
                    // the traversal to happen after
                    buffer.insert_char_n_times(' ', 4);
                    (0..4).into_iter().for_each(|_| {
                        buffer.next_char();
                    });
                    buffer.has_changed = true;
                }
                (KeyCode::Enter, Mode::Insert) => {
                    buffer.insert_char('\n');
                    // buffer.next_char();
                    buffer.cursor += 1;
                    buffer.has_changed = true;
                }

                (KeyCode::Char(c), Mode::Command) => {
                    status_bar.push(c);
                }
                (KeyCode::Enter, Mode::Command) => {
                    for (i, command) in status_bar.iter().enumerate().skip(1) {
                        match command {
                            'w' => match &path {
                                Some(path) => {
                                    io::write(path.to_path_buf(), buffer.clone())?;
                                }
                                None => log("Cannot write buffer, no file opened."),
                            },
                            'l' => {
                                io::load_file(&path, &mut buffer);
                                view_box.flush(&buffer, &status_bar, &mode, &path, false);
                            }
                            'o' => {
                                if status_bar.len() == i + 1 {
                                    break;
                                }
                                let path_buf = PathBuf::from(
                                    status_bar[i + 1..].iter().collect::<String>().trim(),
                                );
                                log(format!("Set path to equal: {}", path_buf.to_string_lossy()));
                                path = Some(path_buf);

                                io::load_file(&path, &mut buffer);
                                view_box.flush(&buffer, &status_bar, &mode, &path, false);

                                break;
                            }
                            'q' => break 'main,
                            's' => {
                                if status_bar[i..].len() == 1 {
                                    break;
                                }
                                let substitution: Vec<&[char]> =
                                    status_bar[i + 1..].split(|c| *c == '/').collect();

                                let original = substitution[0];
                                let new: String = substitution[1].iter().collect();

                                log(format!(
                                    "Substition\n\toriginal: {:?}\n\tnew: {}",
                                    original, new
                                ));

                                let mut curr: Vec<char> = Vec::with_capacity(original.len() - 1);
                                let mut idxs_of_substitution: Vec<usize> = Vec::with_capacity(4);

                                for (i, char) in buffer.rope.chars().enumerate() {
                                    if curr.len() == original.len() {
                                        idxs_of_substitution.push(i);
                                        curr.clear();
                                    }
                                    if char == original[curr.len()] {
                                        curr.push(char);
                                    }
                                }

                                idxs_of_substitution.iter().for_each(|idx| {
                                    buffer.rope.remove(idx - original.len()..*idx);
                                    buffer.rope.insert(idx - original.len(), &new);
                                });
                                break;
                            }
                            n if n.is_numeric() => {
                                let num_str = status_bar[i..].iter().collect::<String>();
                                let num: usize = match num_str.parse() {
                                    Ok(n) => n,
                                    Err(err) => {
                                        log(format!(
                                            "Failed to parse number: {} ({})",
                                            num_str, err
                                        ));
                                        break;
                                    }
                                };

                                buffer.set_row(num + 1);
                            }
                            c => log(format!("Unknown Meta-Command: {}", c)),
                        }
                    }

                    mode = Mode::Normal;
                    status_bar.clear();
                }
                (KeyCode::Esc, Mode::Command) => {
                    mode = Mode::Normal;
                    status_bar.clear();
                }
                (KeyCode::Backspace, Mode::Command) => {
                    status_bar.delete();
                }
                // TODO Update buffer-line
                // Exists to prevent the arrow keys from working for now
                (_, Mode::Command) => {}
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
                    if buffer.is_last_row() {
                        continue;
                    }

                    buffer.next_line();
                    buffer.end_of_line();
                }
                _ => continue,
            };

            let adjusted = view_box.adjust(&buffer);
            view_box.flush(&buffer, &status_bar, &mode, &path, adjusted)?;
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
