use crate::{
    DEBUG, Mode,
    buffer::Buffer,
    log,
    motion::Motion,
    register::RegisterHandler,
    undo::{Action, UndoTree},
};

pub struct Operator<'a> {
    pub name: &'a str,
    command: fn(
        end: usize,
        buffer: &mut Buffer,
        register_handler: &mut RegisterHandler,
        mode: &mut Mode,
        undo_tree: &mut UndoTree,
    ),
}

impl<'a> Operator<'a> {
    pub fn new(
        name: &'a str,
        command: fn(
            end: usize,
            buffer: &mut Buffer,
            register_handler: &mut RegisterHandler,
            mode: &mut Mode,
            undo_tree: &mut UndoTree,
        ),
    ) -> Self {
        Self { name, command }
    }

    pub fn execute(
        &self,
        motion: &Motion,
        buffer: &mut Buffer,
        register_handler: &mut RegisterHandler,
        mode: &mut Mode,
        undo_tree: &mut UndoTree,
    ) {
        let mut end = motion.evaluate(buffer);
        log!("initial end: {end}");
        // There isn't a solution to this
        // There isn't a good way to distinguish between not wanting to delete
        // the first character of the last word and the last character of the last word
        // in the two cases:
        // word c
        // dw--->
        //
        // and
        //
        // word
        // dw->
        if !motion.inclusive {
            // && end != buffer.rope.len_chars() - 1 {
            if end > buffer.cursor {
                end = usize::max(end, 1) - 1;
            } else if end != buffer.rope.len_chars() {
                end += 1;
            }
        }

        (self.command)(end, buffer, register_handler, mode, undo_tree);
    }

    pub fn entire_line(
        &self,
        buffer: &mut Buffer,
        register_handler: &mut RegisterHandler,
        mode: &mut Mode,
        undo_tree: &mut UndoTree,
    ) {
        buffer.start_of_line();
        let end_of_line = buffer.get_end_of_line();
        let reg_before = register_handler.get_reg().to_string();

        (self.command)(end_of_line, buffer, register_handler, mode, undo_tree);
        if reg_before != register_handler.get_reg()
            && register_handler.get_reg().chars().last().unwrap_or('\n') != '\n'
        {
            register_handler.push_reg(&'\n');
        }
    }
}

/// Iterates over a range, allowing an operation to be performed across the range.
/// This is more general than just say, capitalizing every character.
///
/// Thir is how operations like yank and delete are able to work across arbitrary ranges, even
/// ranges defined by the contents of the file, such as 'yfi'
///
/// # Params
/// - `mode`: The current mode the editor is in (eg. Visual, Insert, Command).
/// - `register_handler`: The manager handler, there should just be one for the entire editor.
///   This is necessary for the yanking of items, for example.
/// - `buffer`: The actual underlying text buffer object.
/// - `end`: The index at which to end the traversal on. Must be greater than the initial value of `buffer.cursor`
/// - `initial_callback`: The function which is run before the iteration.
/// - `iter_callback`: The function which runs for every character in the iteration. It should
///   assume that `buffer.cursor` is the current index of the iteration.
///   This is the only callback incapable of accessing the current mode.
/// - `after_callback`: The function which runs after the iteration is complete. This callback is
///   the only one provided with the original index of the cursor before the
///   iteration. One use-case of this callback could be to reset the position of
///   the cursor after the iteration.
///
/// Any of the given callbacks may be noops. Each callback is free modify
pub fn iterate_range(
    mode: &mut Mode,
    register_handler: &mut RegisterHandler,
    buffer: &mut Buffer,
    end: usize,
    initial_callback: fn(
        register_handler: &mut RegisterHandler,
        buffer: &mut Buffer,
        mode: &mut Mode,
    ),
    iter_callback: fn(register_handler: &mut RegisterHandler, buffer: &mut Buffer),
    after_callback: fn(
        start: usize,
        register_handler: &mut RegisterHandler,
        buffer: &mut Buffer,
        mode: &mut Mode,
    ),
    will_delete: bool,
) {
    let anchor = buffer.cursor;
    let count = i32::try_from(end).unwrap() - i32::try_from(anchor).unwrap();
    initial_callback(register_handler, buffer, mode);

    let initial_register_contents = register_handler.get_reg().to_string();

    log!("count: {count}");
    log!("end2: {end}");

    if count.is_positive() {
        if will_delete {
            (0..=count).for_each(|_| iter_callback(register_handler, buffer));
        } else {
            (0..=count).for_each(|_| {
                iter_callback(register_handler, buffer);
                if buffer.cursor + 1 < buffer.rope.len_chars() {
                    buffer.next_char();
                }
            });
        }
    } else {
        (0..=count.abs()).for_each(|_| {
            iter_callback(register_handler, buffer);
            // TODO Probably remove
            // This is just to stop `db` from being weird
            // if c + 1 < count.abs() {
            //     buffer.prev_char();
            // }
        });

        let final_register_contents = register_handler.get_reg();
        if initial_register_contents != final_register_contents {
            register_handler.set_reg(final_register_contents.chars().rev().collect::<String>());
        }
    }

    after_callback(anchor, register_handler, buffer, mode);
}

const fn noop(
    _start: usize,
    _register_handler: &mut RegisterHandler,
    _buffer: &mut Buffer,
    _mode: &mut Mode,
) {
}

const fn reset_position(
    start: usize,
    _register_handler: &mut RegisterHandler,
    buffer: &mut Buffer,
    _mode: &mut Mode,
) {
    buffer.cursor = start;
}

fn insert(
    _start: usize,
    _register_handler: &mut RegisterHandler,
    _buffer: &mut Buffer,
    mode: &mut Mode,
) {
    mode.insert();
}

fn clear_reg(register_handler: &mut RegisterHandler, _buffer: &mut Buffer, _mode: &mut Mode) {
    register_handler.empty_reg();
}

fn delete_char(register_handler: &mut RegisterHandler, buffer: &mut Buffer) {
    if buffer.rope.len_chars() <= buffer.cursor {
        return;
    }
    register_handler.push_reg(&buffer.get_curr_char());
    buffer.delete_curr_char();
}
pub fn delete(
    end: usize,
    buffer: &mut Buffer,
    register_handler: &mut RegisterHandler,
    mode: &mut Mode,
    undo_tree: &mut UndoTree,
) {
    log!("delete end {end}");
    iterate_range(
        mode,
        register_handler,
        buffer,
        end,
        clear_reg,
        delete_char,
        noop,
        true,
    );
    // TODO
    // Currently using the 'd' command across lines will break because of this
    buffer.update_list_use_current_line();

    let text = register_handler.get_reg();
    let action = Action::delete(buffer.cursor, &text);
    undo_tree.new_action(action);

    buffer.cursor = usize::min(buffer.cursor, usize::max(buffer.rope.len_chars(), 1) - 1);
}

fn yank_char(register_handler: &mut RegisterHandler, buffer: &mut Buffer) {
    register_handler.push_reg(&buffer.get_curr_char());
}
pub fn yank(
    end: usize,
    buffer: &mut Buffer,
    register_handler: &mut RegisterHandler,
    mode: &mut Mode,
    _undo_tree: &mut UndoTree,
) {
    if end == buffer.rope.len_chars() && end == buffer.cursor {
        return;
    }

    iterate_range(
        mode,
        register_handler,
        buffer,
        end,
        clear_reg,
        yank_char,
        reset_position,
        false,
    );

    buffer.cursor = usize::min(buffer.cursor, usize::max(buffer.rope.len_chars(), 1) - 1);
}

pub fn change(
    end: usize,
    buffer: &mut Buffer,
    register_handler: &mut RegisterHandler,
    mode: &mut Mode,
    undo_tree: &mut UndoTree,
) {
    iterate_range(
        mode,
        register_handler,
        buffer,
        end,
        clear_reg,
        delete_char,
        insert,
        true,
    );

    // TODO
    // Currently using the 'c' command across lines will break because of this
    buffer.update_list_use_current_line();

    let text = register_handler.get_reg();
    let action = Action::delete(buffer.cursor, &text);
    undo_tree.new_action(action);

    buffer.cursor = usize::min(buffer.cursor, usize::max(buffer.rope.len_chars(), 1) - 1);
}
