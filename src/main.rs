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
mod view;
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
    io::{Cli, log, log_dir, log_file},
    meta_command::match_meta_command,
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
    view::View,
    view_box::{cleanup, setup},
    view_command::{
        ViewCommand, center_viewbox_on_cursor, delete_curr_view_box, move_down_one_view_box,
        move_left_one_view_box, move_right_one_view_box, move_up_one_view_box,
        split_curr_view_box_horizontal, split_curr_view_box_vertical,
    },
};
use anyhow::Result;
use commands::Command as Cmd;
use crossterm::{
    cursor::SetCursorStyle,
    event::{Event, KeyCode, read},
    execute,
    terminal::size,
};
use std::io::{Stdout, stdout};

pub static mut DEBUG: bool = true;

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
    let mut status_bar: StatusBar = StatusBar::new();

    let view_commands: &[ViewCommand] = &[
        ViewCommand::new("zz", center_viewbox_on_cursor),
        // View Box related
        ViewCommand::new("zd", move_down_one_view_box),
        ViewCommand::new("zu", move_up_one_view_box),
        ViewCommand::new("zl", move_left_one_view_box),
        ViewCommand::new("zr", move_right_one_view_box),
        ViewCommand::new("zx", delete_curr_view_box),
        ViewCommand::new("zv", split_curr_view_box_vertical),
        ViewCommand::new("zh", split_curr_view_box_horizontal),
    ];

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
    let mut view: View = View::new(cols, rows);
    setup(rows, cols)?;

    let mut mode = Mode::Normal;
    let mut count: u16 = 1;
    let mut chained: Vec<char> = vec![];

    let (cli, path) = Cli::parse_path()?;
    unsafe {
        DEBUG = cli.debug;
    }

    view.set_path(path);
    view.load_file()?;

    view.flush(
        &status_bar,
        &mode,
        &chained,
        count,
        register_handler.get_curr_reg(),
        false,
    )?;

    program_loop(
        commands,
        operators,
        motions,
        view_commands,
        &mut count,
        &mut chained,
        &mut next_operation,
        &all_normal_chars,
        &mut stdout,
        &mut status_bar,
        &mut register_handler,
        &mut undo_tree,
        &mut view,
        &mut mode,
    )?;

    cleanup()
}

// TODO
// I'm pretty sure this function can just consume all its arguments
// But I wasn't thinking about it at the time and now I'm too lazy to change it
//
/// The main loop of Orinfar
/// Essentially just waits for a keypress, matches on it, then updates the state of the editor in
/// accordance with the action taken.
/// # Arguments
/// This function essentially consumes every relevant piece of data in the program
#[allow(clippy::too_many_lines)]
#[allow(clippy::too_many_arguments)]
fn program_loop<'a>(
    commands: &[Cmd],
    operators: &'a [Operator<'a>],
    motions: &[Motion],
    view_commands: &[ViewCommand],

    count: &mut u16,
    chained: &mut Vec<char>,
    next_operation: &mut Option<&'a Operator<'a>>,
    all_normal_chars: &[char],

    stdout: &mut Stdout,

    status_bar: &mut StatusBar,
    register_handler: &mut RegisterHandler,
    undo_tree: &mut UndoTree,
    view: &mut View,
    mode: &mut Mode,
) -> Result<()> {
    'main: loop {
        let buffer = view.get_buffer_mut();
        buffer.update_list_reset();

        if let Event::Key(event) = read()? {
            match (event.code, mode.clone()) {
                (KeyCode::Char(c), Mode::Normal) if c.is_numeric() => {
                    let c = u16::try_from(c.to_digit(10).expect("Numeric digit not in base 10"))
                        .unwrap();
                    if *count == 1 {
                        *count = 0;
                    }
                    *count *= 10;
                    *count += c;
                }
                (KeyCode::Char(':'), Mode::Normal) => {
                    *mode = Mode::Meta;
                    status_bar.push(':');
                }

                (KeyCode::Char(c), Mode::Normal) => {
                    match_action(
                        c,
                        chained,
                        next_operation,
                        count,
                        register_handler,
                        undo_tree,
                        view,
                        mode,
                        commands,
                        operators,
                        motions,
                        view_commands,
                        all_normal_chars,
                    );
                }

                (KeyCode::Esc, Mode::Normal) => {
                    chained.clear();
                    *count = 1;
                    *next_operation = None;
                }
                (KeyCode::Esc, Mode::Insert) => {
                    if buffer.cursor != buffer.get_start_of_line() {
                        buffer.cursor -= 1;
                    }
                    *mode = Mode::Normal;
                    execute!(stdout, SetCursorStyle::SteadyBlock)?;
                }
                (KeyCode::Backspace, Mode::Insert) => {
                    buffer.backspace(undo_tree);
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
                    if match_meta_command(status_bar, view, register_handler, undo_tree, mode)? {
                        break 'main;
                    }
                }
                (KeyCode::Esc, Mode::Meta) => {
                    *mode = Mode::Normal;
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
                    prev_row(buffer);
                }
                (KeyCode::Down, _) => {
                    next_row(buffer);
                }
                _ => continue,
            }

            let adjusted = view.adjust();
            view.flush(
                status_bar,
                mode,
                chained,
                *count,
                register_handler.get_curr_reg(),
                adjusted,
            )?;
        }
    }

    Ok(())
}

/// # Errors
/// - I/O error if `crossterm::events::read()` fails
pub fn on_next_input(buffer: &mut Buffer, callback: fn(KeyCode, &mut Buffer)) -> Result<()> {
    loop {
        if let Event::Key(event) = read()? {
            callback(event.code, buffer);
            break;
        }
    }

    Ok(())
}
