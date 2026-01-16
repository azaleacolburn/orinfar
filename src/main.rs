#![feature(panic_update_hook)]

// Needs to be defined first
#[macro_use]
mod utility;
mod action;
mod buffer;
mod buffer_char;
mod buffer_line;
mod buffer_update;
mod commands;
mod meta_command;
#[macro_use]
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
    action::{enumerate_normal_chars, match_action},
    buffer::Buffer,
    commands::{
        append, cut, first_row, insert, insert_new_line, insert_new_line_above, last_row, paste,
        replace, set_curr_register, undo,
    },
    io::{Cli, log, log_dir, log_file, try_get_git_hash},
    meta_command::{attach_buffer, print_directories, substitute_cmd},
    mode::Mode,
    motion::{
        Motion, back, beginning_of_line, end_of_line, end_of_word, find, find_back, find_until,
        next_char, next_corresponding_bracket, next_newline, next_row, prev_char, prev_newline,
        prev_row, word,
    },
    operator::{Operator, change, delete, yank},
    register::RegisterHandler,
    status_bar::StatusBar,
    undo::{Action, UndoTree},
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
use std::io::stdout;

pub static mut DEBUG: bool = true;

#[allow(clippy::too_many_lines)]
fn main() -> Result<()> {
    panic_hook::add_panic_hook(&cleanup);

    // This could fail if the dir already exists, so we don't care if this fails
    if let Err(err) = std::fs::create_dir(log_dir())
        && err.to_string() != "File exists (os error 17)"
    {
        return Err(err.into());
    }
    std::fs::File::create(log_file())?;

    let mut stdout = stdout();

    let mut undo_tree = UndoTree::new();
    let mut register_handler = RegisterHandler::new();
    let mut buffer: Buffer = Buffer::new();
    let mut status_bar: StatusBar = StatusBar::new();

    let view_commands: &[ViewCommand] = &[ViewCommand::new("zz", center_viewbox_on_cursor)];
    let commands: &[Cmd] = &[
        // Insert
        Cmd::new("i", insert),
        Cmd::new("a", append),
        Cmd::new("o", insert_new_line),
        Cmd::new("O", insert_new_line_above),
        // Single character edit
        Cmd::new("x", cut),
        Cmd::new("r", replace),
        // File Traversal
        Cmd::new("G", last_row),
        Cmd::new("gg", first_row),
        // Misc
        Cmd::new("u", undo),
        Cmd::new("p", paste),
        Cmd::new("\"", set_curr_register),
    ];
    let operators: &[Operator] = &[
        Operator::new("d", delete),
        Operator::new("y", yank),
        Operator::new("c", change),
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
        // Line operators
        Motion::inclusive("$", end_of_line),
        Motion::inclusive("_", beginning_of_line),
        // Finding operators
        Motion::inclusive("f", find),
        Motion::inclusive("F", find_back),
        Motion::inclusive("t", find_until),
        // Paragraph operators
        Motion::inclusive("%", next_corresponding_bracket),
        Motion::inclusive("}", next_newline),
        Motion::inclusive("{", prev_newline),
    ];
    let mut next_operation: Option<&Operator> = None;

    // Used for not putting excluded chars in the chain
    let all_normal_chars = enumerate_normal_chars(commands, operators, motions, view_commands);

    let (cols, rows) = size()?;
    let mut view_box: ViewBox = ViewBox::new(cols, rows);
    setup(rows, cols)?;

    let mut mode = Mode::Normal;
    let mut count: u16 = 1;
    let mut chained: Vec<char> = vec![];

    let (cli, mut path) = Cli::parse_path()?;
    unsafe {
        DEBUG = cli.debug;
    }

    let mut git_hash = try_get_git_hash(path.as_ref());
    io::load_file(path.as_ref(), &mut buffer)?;

    view_box.flush(
        &buffer,
        &status_bar,
        &mode,
        &chained,
        count,
        register_handler.get_curr_reg(),
        path.as_ref(),
        git_hash.as_deref(),
        false,
    )?;

    'main: loop {
        buffer.update_list_reset();

        if let Event::Key(event) = read()? {
            match (event.code, mode.clone()) {
                (KeyCode::Char(c), Mode::Normal) if c.is_numeric() => {
                    let c = u16::try_from(c.to_digit(10).expect("Numeric digit not in base 10"))
                        .unwrap();
                    if count == 1 {
                        count = 0;
                    }
                    count *= 10;
                    count += c;
                }
                (KeyCode::Char(':'), Mode::Normal) => {
                    mode = Mode::Meta;
                    status_bar.push(':');
                }

                (KeyCode::Char(c), Mode::Normal) => {
                    match_action(
                        c,
                        &mut chained,
                        &mut next_operation,
                        &mut count,
                        &mut buffer,
                        &mut register_handler,
                        &mut undo_tree,
                        &mut view_box,
                        &mut mode,
                        commands,
                        operators,
                        motions,
                        view_commands,
                        &all_normal_chars,
                    );
                }

                (KeyCode::Esc, Mode::Normal) => {
                    chained.clear();
                    count = 1;
                    next_operation = None;
                }
                (KeyCode::Esc, Mode::Insert) => {
                    if buffer.cursor != buffer.get_start_of_line() {
                        buffer.cursor -= 1;
                    }
                    mode = Mode::Normal;
                    execute!(stdout, SetCursorStyle::SteadyBlock)?;
                }
                (KeyCode::Backspace, Mode::Insert) => {
                    buffer.backspace(&mut undo_tree);
                }
                (KeyCode::Char(c), Mode::Insert) => {
                    buffer.insert_char(c);
                    buffer.cursor += 1;
                    buffer.update_list_use_current_line();

                    let action = Action::insert(buffer.cursor - 1, &c);
                    undo_tree.new_action_merge(action);
                }
                (KeyCode::Tab, Mode::Insert) => {
                    // NOTE
                    // Iterates two separate times because we want the insertation batched and
                    // the traversal to happen after
                    buffer.insert_char_n_times(' ', 4);
                    (0..4).for_each(|_| {
                        buffer.next_char();
                    });
                    buffer.update_list_use_current_line();
                }
                (KeyCode::Enter, Mode::Insert) => {
                    let newline = buffer.insert_newline();

                    let action = Action::insert(buffer.cursor - newline.len(), &newline);
                    undo_tree.new_action(action);
                }

                (KeyCode::Char(c), Mode::Meta) => {
                    status_bar.push(c);
                }
                (KeyCode::Enter, Mode::Meta) => {
                    for (i, command) in status_bar.iter().enumerate().skip(1) {
                        match command {
                            'w' => match &path {
                                Some(path) => {
                                    io::write(path.clone(), &buffer)?;
                                }
                                None => log!("WARNING: Cannot Write Unattached Buffer"),
                            },
                            'u' => {
                                path = None;
                            }
                            'l' => {
                                io::load_file(path.as_ref(), &mut buffer)?;
                                view_box.flush(
                                    &buffer,
                                    &status_bar,
                                    &mode,
                                    &chained,
                                    count,
                                    register_handler.get_curr_reg(),
                                    path.as_ref(),
                                    git_hash.as_deref(),
                                    false,
                                )?;
                            }
                            'o' => {
                                attach_buffer(
                                    &mut buffer,
                                    &status_bar,
                                    i,
                                    &mut path,
                                    &mut git_hash,
                                );

                                io::load_file(path.as_ref(), &mut buffer)?;
                                view_box.flush(
                                    &buffer,
                                    &status_bar,
                                    &mode,
                                    &chained,
                                    count,
                                    register_handler.get_curr_reg(),
                                    path.as_ref(),
                                    git_hash.as_deref(),
                                    false,
                                )?;
                                break;
                            }
                            'd' => {
                                print_directories(&mut buffer, &mut undo_tree, path.clone())?;
                            }
                            // Print Registers
                            'r' => {
                                let contents = register_handler.to_string();
                                buffer.replace_contents(contents, &mut undo_tree);
                            }
                            's' => {
                                substitute_cmd(&mut buffer, &status_bar, &mut undo_tree, i);
                                break;
                            }
                            n if n.is_numeric() => {
                                let num_str = status_bar[i..].iter().collect::<String>();
                                let num: usize = match num_str.parse() {
                                    Ok(n) => n,
                                    Err(err) => {
                                        log!("Failed to parse number: {} ({})", num_str, err);
                                        break;
                                    }
                                };

                                buffer.set_row(num + 1);
                            }
                            'q' => break 'main,
                            c => log!("Unknown Meta-Command: {}", c),
                        }
                    }

                    mode = Mode::Normal;
                    status_bar.clear();
                }
                (KeyCode::Esc, Mode::Meta) => {
                    mode = Mode::Normal;
                    status_bar.clear();
                }
                (KeyCode::Backspace, Mode::Meta) => {
                    status_bar.delete();
                }
                // TODO Update buffer-line
                // Exists to prevent the arrow keys from working for now
                (_, Mode::Meta) => {}
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
            }

            let adjusted = view_box.adjust(&mut buffer);
            view_box.flush(
                &buffer,
                &status_bar,
                &mode,
                &chained,
                count,
                register_handler.get_curr_reg(),
                path.as_ref(),
                git_hash.as_deref(),
                adjusted,
            )?;
        }
    }

    cleanup()
}

/// # Errors
/// - I/O error if `crossterm::events::read()` fails
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

/// # Errors
/// - I/O error if `crossterm::events::read()` fails
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
