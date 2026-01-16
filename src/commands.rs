use crate::{
    DEBUG, Mode,
    buffer::Buffer,
    log,
    register::RegisterHandler,
    undo::{Action, UndoTree},
};
use crossterm::{
    cursor::SetCursorStyle,
    event::{Event, KeyCode, read},
    execute,
};
use std::io::stdout;

pub struct Command<'a> {
    pub name: &'a str,
    command: fn(
        buffer: &mut Buffer,
        register_handler: &mut RegisterHandler,
        mode: &mut Mode,
        undo_tree: &mut UndoTree,
    ),
}

impl<'a> Command<'a> {
    pub fn new(
        name: &'a str,
        command: fn(
            buffer: &mut Buffer,
            register_handler: &mut RegisterHandler,
            mode: &mut Mode,

            undo_tree: &mut UndoTree,
        ),
    ) -> Self {
        Command { name, command }
    }

    pub fn execute(
        &self,
        buffer: &mut Buffer,
        register_handler: &mut RegisterHandler,
        mode: &mut Mode,
        undo_tree: &mut UndoTree,
    ) {
        (self.command)(buffer, register_handler, mode, undo_tree);
    }
}

pub fn insert(
    _buffer: &mut Buffer,
    _register_handler: &mut RegisterHandler,
    mode: &mut Mode,
    _undo_tree: &mut UndoTree,
) {
    mode.insert();
}

pub fn append(
    buffer: &mut Buffer,
    _register_handler: &mut RegisterHandler,
    mode: &mut Mode,
    _undo_tree: &mut UndoTree,
) {
    if buffer.cursor != buffer.get_end_of_line() || buffer.cursor + 1 == buffer.rope.len_chars() {
        buffer.cursor += 1;
    }

    mode.insert();
}

pub fn cut(
    buffer: &mut Buffer,
    register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
    undo_tree: &mut UndoTree,
) {
    if let Some(c) = buffer.rope.get_char(buffer.cursor) {
        if c == '\n' {
            return;
        }
        register_handler.set_reg(c.to_string());
        let anchor = buffer.cursor;

        buffer.rope.remove(buffer.cursor..=buffer.cursor);
        if buffer.cursor != 0 && buffer.is_last_col() {
            buffer.cursor -= 1;
        }
        buffer.update_list_use_current_line();

        let action = Action::delete(anchor, &c);
        undo_tree.new_action(action);
    }
}

/// Acts like 'o' does in my nixvim config, it inserts spaces at the beginning of the line to align
/// the first non-whitespace line of the new line with that of the old one
pub fn insert_new_line(
    buffer: &mut Buffer,
    register_handler: &mut RegisterHandler,
    mode: &mut Mode,
    undo_tree: &mut UndoTree,
) {
    buffer.end_of_line();
    let anchor = buffer.cursor;
    append(buffer, register_handler, mode, undo_tree);
    let newline = buffer.insert_newline();

    let action = Action::insert(anchor, &newline);
    undo_tree.new_action(action);
}

pub fn insert_new_line_above(
    buffer: &mut Buffer,
    _register_handler: &mut RegisterHandler,
    mode: &mut Mode,
    undo_tree: &mut UndoTree,
) {
    let line_idx = buffer.rope.char_to_line(buffer.cursor);
    let first = buffer.rope.line_to_char(line_idx);
    buffer.update_list_add(line_idx);
    buffer.rope.insert_char(first, '\n');

    buffer.cursor = first;
    mode.insert();
    buffer.has_changed = true;

    let action = Action::insert(first, &'\n');
    undo_tree.new_action(action);
}

pub fn paste(
    buffer: &mut Buffer,
    register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
    undo_tree: &mut UndoTree,
) {
    log!(
        "printing contents: {:?}\ncurrent register contents{}\n",
        register_handler.to_string(),
        register_handler.get_curr_reg(),
    );
    let contents = &register_handler.get_reg();

    let line_idx = buffer.get_row();
    contents
        .chars()
        .filter(|c| *c == '\n')
        .for_each(|_| buffer.update_list_add(line_idx));

    buffer.rope.insert(buffer.cursor, contents);
    buffer.update_list_use_current_line();

    let action = Action::insert(buffer.cursor, contents);
    undo_tree.new_action(action);
}

pub fn _crash(
    _buffer: &mut Buffer,
    _register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
    _undo_tree: &mut UndoTree,
) {
    panic!("Intentionally Crashed")
}

pub fn set_curr_register(
    _buffer: &mut Buffer,
    register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
    _undo_tree: &mut UndoTree,
) {
    if let Event::Key(event) = read().expect("Could not read character from stdin")
        && let KeyCode::Char(reg_name) = event.code
    {
        register_handler.current_register = reg_name;
    }
}

pub fn replace(
    buffer: &mut Buffer,
    _register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
    undo_tree: &mut UndoTree,
) {
    if buffer.cursor == buffer.rope.len_chars() {
        return;
    }
    execute!(stdout(), SetCursorStyle::SteadyUnderScore)
        .expect("Crossterm steady underscore command failed");
    if let Event::Key(event) = read().expect("Failed to read replacement character")
        && let KeyCode::Char(c) = event.code
    {
        let original_char = buffer.get_curr_char();
        buffer.replace_curr_char(c);

        let action = Action::replace(vec![buffer.cursor + 1], &original_char, &c);
        undo_tree.new_action_merge(action);
    }
    execute!(stdout(), SetCursorStyle::SteadyBlock).expect("Crossterm steady block command failed");
    buffer.has_changed = true;
}

pub fn undo(
    buffer: &mut Buffer,
    _register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
    undo_tree: &mut UndoTree,
) {
    undo_tree.undo(buffer);
}

pub fn last_row(
    buffer: &mut Buffer,
    _register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
    _undo_tree: &mut UndoTree,
) {
    let last_row = usize::max(buffer.len(), 1) - 1;
    buffer.set_row(last_row);
}

pub fn first_row(
    buffer: &mut Buffer,
    _register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
    _undo_tree: &mut UndoTree,
) {
    buffer.set_row(0);
}
