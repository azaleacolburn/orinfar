use crate::{
    DEBUG,
    action::match_action,
    commands::Command as Cmd,
    io::log,
    meta_command::match_meta_command,
    mode::Mode,
    motion::{Motion, next_row, prev_row},
    operator::Operator,
    register::RegisterHandler,
    status_bar::StatusBar,
    text_object::{TextObject, TextObjectType},
    undo::{Action, UndoTree},
    view::View,
    view_command::ViewCommand,
};
use anyhow::Result;
use crossterm::event::{Event, KeyCode, read};

/// The main loop of Orinfar
/// Essentially just waits for a keypress, matches on it, then updates the state of the editor in
/// accordance with the action taken.
/// # Arguments
/// This function essentially consumes every relevant piece of data in the program
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
pub fn program_loop<'a>(
    commands: &[Cmd],
    operators: &'a [Operator<'a>],
    motions: &[Motion],
    text_objects: &[TextObject],
    view_commands: &[ViewCommand],

    mut count: u16,
    mut chained: Vec<char>,
    mut next_operation: Option<&'a Operator<'a>>,
    mut text_object_type: Option<TextObjectType>,
    all_normal_chars: &[char],

    mut search_str: Vec<char>,

    mut status_bar: StatusBar,
    mut register_handler: RegisterHandler,
    mut undo_tree: UndoTree,
    mut view: View,
    mut mode: Mode,
) -> Result<()> {
    let mut last_count = 1;
    let mut last_chained: Vec<char> = vec![];

    'main: loop {
        let buffer = view.get_buffer_mut();
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
                (KeyCode::Char('/'), Mode::Normal) => {
                    mode.search();
                    status_bar.push('/');
                }
                (KeyCode::Char('n'), Mode::Normal) => buffer.goto_next_string(&search_str),
                (KeyCode::Char('N'), Mode::Normal) => buffer.goto_prev_string(&search_str),
                (KeyCode::Char('.'), Mode::Normal) => match_action(
                    &mut chained,
                    &mut next_operation,
                    &mut text_object_type,
                    &mut count,
                    &mut last_chained,
                    &mut last_count,
                    &mut register_handler,
                    &mut undo_tree,
                    &mut view,
                    &mut mode,
                    commands,
                    operators,
                    motions,
                    text_objects,
                    view_commands,
                ),
                (KeyCode::Char(c), Mode::Normal) => {
                    log!("c {}", c);
                    if !all_normal_chars.contains(&c) {
                        continue;
                    }
                    chained.push(c);

                    match_action(
                        &mut chained,
                        &mut next_operation,
                        &mut text_object_type,
                        &mut count,
                        &mut last_chained,
                        &mut last_count,
                        &mut register_handler,
                        &mut undo_tree,
                        &mut view,
                        &mut mode,
                        commands,
                        operators,
                        motions,
                        text_objects,
                        view_commands,
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
                    mode.normal();
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

                (KeyCode::Enter, Mode::Meta) => {
                    if match_meta_command(
                        &mut status_bar,
                        &mut view,
                        &register_handler,
                        &mut undo_tree,
                        &mut mode,
                    )? {
                        break 'main;
                    }
                }

                (KeyCode::Char(c), Mode::Meta | Mode::Search) => {
                    status_bar.push(c);
                }
                (KeyCode::Esc, Mode::Meta | Mode::Search) => {
                    mode.normal();
                    status_bar.clear();
                }
                (KeyCode::Backspace, Mode::Meta | Mode::Search) => {
                    status_bar.delete();
                }
                (_, Mode::Meta) => {}

                (KeyCode::Enter, Mode::Search) => {
                    search_str = status_bar.buffer().split_at(1).1.chars().collect();
                    mode.normal();
                    status_bar.clear();
                }

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

            let _ = view.get_view_box().parse();

            let adjusted = view.adjust();
            view.flush(
                &status_bar,
                &mode,
                &chained,
                count,
                register_handler.get_curr_reg(),
                adjusted,
            )?;
        }
    }

    Ok(())
}
