#![feature(panic_update_hook)]

// Needs to be defined first
#[macro_use]
mod utility;
mod buffer;
mod buffer_char;
mod buffer_line;
mod buffer_update;
mod commands;
mod io;
mod mode;
mod motion;
mod operator;
mod panic_hook;
mod register;
mod status_bar;
mod undo;
mod view_box;
mod view_command;

use crate::{
    buffer::Buffer,
    commands::{
        append, cut, first_row, insert, insert_new_line, insert_new_line_above, last_row, paste,
        replace, undo,
    },
    io::Cli,
    mode::Mode,
    motion::{
        Motion, back, beginning_of_line, end_of_line, end_of_word, find, find_back, next_char,
        next_corresponding_bracket, next_newline, next_row, prev_char, prev_newline, prev_row,
        word,
    },
    operator::{Operator, change, change_until_before, delete, yank},
    register::RegisterHandler,
    status_bar::StatusBar,
    undo::{Action, UndoTree},
    utility::log,
    view_box::{ViewBox, cleanup, setup},
    view_command::{ViewCommand, center_viewbox_on_cursor},
};
use anyhow::Result;
use commands::Command as Cmd;
use crossterm::{
    cursor::SetCursorStyle,
    event::{Event, KeyCode, read},
    execute,
    terminal::size,
};
use std::{io::stdout, path::PathBuf, u16};

fn main() -> Result<()> {
    panic_hook::add_panic_hook(&cleanup);

    std::fs::File::create("log.txt")?;

    let mut stdout = stdout();
    let _leader = ' ';

    let mut undo_tree = UndoTree::new();
    let mut register_handler = RegisterHandler::new();
    let mut buffer: Buffer = Buffer::new();
    let mut status_bar: StatusBar = StatusBar::new();

    let view_commands: &[ViewCommand] = &[ViewCommand::new("zz", center_viewbox_on_cursor)];
    let commands: &[Cmd] = &[
        Cmd::new("i", insert),
        Cmd::new("p", paste),
        Cmd::new("a", append),
        Cmd::new("o", insert_new_line),
        Cmd::new("O", insert_new_line_above),
        Cmd::new("x", cut),
        Cmd::new("r", replace),
        Cmd::new("G", last_row),
        Cmd::new("gg", first_row),
        Cmd::new("u", undo),
    ];
    let operators: &[Operator] = &[
        Operator::new("d", delete),
        Operator::new("y", yank),
        Operator::new("c", change),
        Operator::new("t", change_until_before),
    ];
    let motions: &[Motion] = &[
        // HJKL
        Motion::inclusive("h", prev_char),
        Motion::inclusive("j", next_row),
        Motion::inclusive("k", prev_row),
        Motion::inclusive("l", next_char),
        // Word operators
        Motion::exclusive("w", word),
        Motion::exclusive("b", back),
        Motion::inclusive("e", end_of_word),
        Motion::inclusive("$", end_of_line),
        Motion::inclusive("_", beginning_of_line),
        Motion::inclusive("f", find),
        Motion::inclusive("F", find_back),
        Motion::inclusive("%", next_corresponding_bracket),
        Motion::inclusive("}", next_newline),
        Motion::inclusive("{", prev_newline),
    ];
    let mut next_operation: Option<&Operator> = None;

    // Used for not putting excluded chars in the chain
    let command_chars = commands.iter().map(|cmd| cmd.name.chars()).flatten();
    let operator_chars = operators.iter().map(|cmd| cmd.name.chars()).flatten();
    let motion_chars = motions.iter().map(|cmd| cmd.name.chars()).flatten();
    let view_command_chars = view_commands.iter().map(|cmd| cmd.name.chars()).flatten();
    let all_normal_chars: Vec<char> = command_chars
        .chain(operator_chars)
        .chain(motion_chars)
        .chain(view_command_chars)
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
        buffer.update_list_reset();

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

                    if let Some(command) = commands
                        .iter()
                        .find(|motion| motion.name == chained.iter().collect::<String>())
                    {
                        command.execute(
                            &mut buffer,
                            &mut register_handler,
                            &mut mode,
                            &mut undo_tree,
                        );
                        chained.clear();
                    } else if let Some(view_command) = view_commands
                        .iter()
                        .find(|command| command.name == chained.iter().collect::<String>())
                    {
                        view_command.execute(&mut buffer, &mut view_box);
                        chained.clear();
                    } else if let Some(operation) = next_operation {
                        if let Some(motion) = motions
                            .iter()
                            .find(|motion| motion.name.chars().nth(0).unwrap() == c)
                        {
                            operation.execute(
                                motion,
                                &mut buffer,
                                &mut register_handler,
                                &mut mode,
                                &mut undo_tree,
                            );
                            chained.clear();
                            next_operation = None;
                        } else if c == operation.name.chars().nth(0).unwrap() {
                            operation.entire_line(
                                &mut buffer,
                                &mut register_handler,
                                &mut mode,
                                &mut undo_tree,
                            );
                            chained.clear();
                            next_operation = None;
                        }
                    } else if chained.len() == 1 {
                        if let Some(motion) = motions
                            .iter()
                            .find(|motion| motion.name.chars().nth(0).unwrap() == c)
                        {
                            motion.apply(&mut buffer);
                            chained.clear();
                        }
                    }
                    if let Some(operator) = operators
                        .iter()
                        .find(|operator| operator.name == chained.iter().collect::<String>())
                    {
                        next_operation = Some(&operator);
                    }
                }

                (KeyCode::Esc, Mode::Insert) => {
                    if buffer.cursor != buffer.get_start_of_line() {
                        buffer.cursor -= 1;
                    }
                    mode = Mode::Normal;
                    execute!(stdout, SetCursorStyle::SteadyBlock)?;
                }
                (KeyCode::Backspace, Mode::Insert) => {
                    if buffer.cursor == 0 {
                        continue;
                    }
                    buffer.cursor -= 1;
                    let char = buffer.get_curr_char();
                    buffer.delete_curr_char();
                    buffer.update_list_use_current_line();

                    let action = Action::delete(buffer.cursor, char);
                    undo_tree.new_action(action);
                }
                (KeyCode::Char(c), Mode::Insert) => {
                    buffer.insert_char(c);
                    // if buffer.rope.len_chars() > 1 {
                    buffer.cursor += 1;
                    buffer.update_list_use_current_line();

                    let action = Action::insert(buffer.cursor - 1, c);
                    undo_tree.new_action(action);
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
                    buffer.update_list_use_current_line();
                }
                (KeyCode::Enter, Mode::Insert) => {
                    buffer.insert_char('\n');
                    // buffer.next_char();
                    buffer.cursor += 1;
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
                    prev_row(&mut buffer);
                }
                (KeyCode::Down, _) => {
                    next_row(&mut buffer);
                }
                _ => continue,
            };

            let adjusted = view_box.adjust(&mut buffer);
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
