use crate::{
    action::match_action, commands::Command as Cmd, count::change_count, global_state::GlobalState,
    meta_command::match_meta_command, mode::Mode, motion::Motion, operator::Operator,
    shell_commands, text_object::TextObject, undo::Action, view::View, view_command::ViewCommand,
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
    all_normal_chars: &[char],

    mut global_state: GlobalState<'a>,
    mut view: View,
) -> Result<()> {
    let mut last_count = 1;
    let mut last_chained: Vec<char> = vec![];

    'main: loop {
        let buffer = view.get_buffer_mut();
        buffer.update_list_reset();

        if let Event::Key(event) = read()? {
            match (event.code, global_state.mode.clone()) {
                (KeyCode::Char(c), Mode::Normal) if c.is_numeric() => {
                    change_count(c, &mut global_state.count);
                }

                (KeyCode::Char(':'), Mode::Normal) => {
                    global_state.mode = Mode::Meta;
                    global_state.status_bar.push(':');
                }
                (KeyCode::Char('#'), Mode::Normal) => {
                    global_state.mode = Mode::Shell;
                    global_state.status_bar.push('#');
                }
                (KeyCode::Char('/'), Mode::Normal) => {
                    global_state.mode.search();
                    global_state.status_bar.push('/');
                }
                (KeyCode::Char('n'), Mode::Normal) => {
                    buffer.goto_next_string(&global_state.search_str);
                }
                (KeyCode::Char('N'), Mode::Normal) => {
                    buffer.goto_prev_string(&global_state.search_str);
                }
                (KeyCode::Char('.'), Mode::Normal) => match_action(
                    &mut global_state,
                    &mut last_chained,
                    &mut last_count,
                    &mut view,
                    commands,
                    operators,
                    motions,
                    text_objects,
                    view_commands,
                ),
                (KeyCode::Char(c), Mode::Normal) => {
                    if !all_normal_chars.contains(&c) {
                        continue;
                    }
                    global_state.chained.push(c);

                    match_action(
                        &mut global_state,
                        &mut last_chained,
                        &mut last_count,
                        &mut view,
                        commands,
                        operators,
                        motions,
                        text_objects,
                        view_commands,
                    );
                }

                (KeyCode::Esc, Mode::Normal) => {
                    global_state.chained.clear();
                    global_state.count = 1;
                    global_state.next_operation = None;
                }
                (KeyCode::Esc, Mode::Insert) => {
                    if buffer.cursor != buffer.get_start_of_line() {
                        buffer.cursor -= 1;
                    }
                    global_state.mode.normal();
                }
                (KeyCode::Backspace, Mode::Insert) => {
                    buffer.backspace(&mut global_state.undo_tree);
                }
                (KeyCode::Char(c), Mode::Insert) => {
                    buffer.insert_char(c);
                    buffer.cursor += 1;
                    buffer.update_list_use_current_line();

                    let action = Action::insert(buffer.cursor - 1, &c);
                    global_state.undo_tree.new_action_merge(action);
                }
                (KeyCode::Tab, Mode::Insert) => {
                    buffer.insert_n_times(' ', 4);
                    buffer.cursor += 4;

                    buffer.update_list_use_current_line();
                }
                (KeyCode::Enter, Mode::Insert) => {
                    let newline = buffer.insert_newline();

                    let action = Action::insert(buffer.cursor - newline.len(), &newline);
                    global_state.undo_tree.new_action(action);
                }

                (KeyCode::Enter, Mode::Meta) => {
                    if match_meta_command(&mut global_state, &mut view)? {
                        break 'main;
                    }
                }
                (KeyCode::Enter, Mode::Shell) => {
                    let _ = shell_commands::command(
                        &mut global_state.status_bar,
                        &mut global_state.mode,
                    );
                }

                (KeyCode::Char(c), Mode::Meta | Mode::Search | Mode::Shell) => {
                    global_state.status_bar.push(c);
                }
                (KeyCode::Esc, Mode::Meta | Mode::Search | Mode::Shell) => {
                    global_state.mode.normal();
                    global_state.status_bar.clear();
                }
                (KeyCode::Backspace, Mode::Meta | Mode::Search | Mode::Shell) => {
                    global_state.status_bar.delete();
                }
                (_, Mode::Meta | Mode::Shell) => {}

                (KeyCode::Enter, Mode::Search) => {
                    global_state.search_str = global_state
                        .status_bar
                        .buffer()
                        .split_at(1)
                        .1
                        .chars()
                        .collect();
                    global_state.mode.normal();
                    global_state.status_bar.clear();
                }

                (KeyCode::Left, _) => buffer.prev_char(),
                (KeyCode::Right, _) => buffer.next_char(),
                (KeyCode::Up, _) => buffer.prev_row(),
                (KeyCode::Down, _) => buffer.next_row(),

                _ => continue,
            }

            let _ = view.get_view_box().parse();

            let adjusted = view.adjust();
            view.flush(&global_state, adjusted)?;
        }
    }

    Ok(())
}
