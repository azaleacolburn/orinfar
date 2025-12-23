use crate::{buffer::Buffer, motion::Motion, register::RegisterHandler, Mode};

pub struct Operator<'a> {
    pub name: &'a [char],
    command: fn(
        end: usize,
        buffer: &mut Buffer,
        register_handler: &mut RegisterHandler,
        mode: &mut Mode,
    ),
}

impl<'a> Operator<'a> {
    pub fn new(
        name: &'a [char],
        command: fn(
            end: usize,
            buffer: &mut Buffer,
            register_handler: &mut RegisterHandler,
            mode: &mut Mode,
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
    ) {
        let end = motion.evaluate(buffer);
        (self.command)(end, buffer, register_handler, mode);
    }

    pub fn entire_line(
        &self,
        buffer: &mut Buffer,
        register_handler: &mut RegisterHandler,
        mode: &mut Mode,
    ) {
        let anchor = buffer.cursor;
        let len = buffer.get_curr_line().len_chars();

        (self.command)(buffer.cursor - 1, buffer, register_handler, mode);
        buffer.cursor = usize::min(anchor, len - 1);
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
///                       This is necessary for the yanking of items, for example.
/// - `buffer`: The actual underlying text buffer object.
/// - `end`: The index at which to end the traversal on. Must be greater than the initial value of `buffer.cursor`
/// - `initial_callback`: The function which is run before the iteration.
/// - `iter_callback`: The function which runs for every character in the iteration. It should
///                    assume that `buffer.cursor` is the current index of the iteration.
///                    This is the only callback incapable of accessing the current mode.
/// - `after_callback`: The function which runs after the iteration is complete. This callback is
///                     the only one provided with the original index of the cursor before the
///                     iteration. One use-case of this callback could be to reset the position of
///                     the cursor after the iteration.
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
) {
    // NOTE We can't actually know the distance between cursors
    // without traversing the buffer since lines are of arbitrary length
    let mut count: usize = 0;
    let anchor_cursor = buffer.cursor.clone();
    while buffer.cursor != end {
        buffer.next_char();
        count += 1;
    }
    buffer.cursor = anchor_cursor.clone();

    initial_callback(register_handler, buffer, mode);
    (0..=count)
        .into_iter()
        .for_each(|_| iter_callback(register_handler, buffer));
    after_callback(anchor_cursor, register_handler, buffer, mode);
}

fn noop(
    _start: usize,
    _register_handler: &mut RegisterHandler,
    _buffer: &mut Buffer,
    _mode: &mut Mode,
) {
}

fn reset_position(
    start: usize,
    _register_handler: &mut RegisterHandler,
    buffer: &mut Buffer,
    _mode: &mut Mode,
) {
    buffer.cursor = start
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
    register_handler.push_reg(&buffer.get_curr_char().to_string());
    buffer.delete_curr_char();
}
pub fn delete(
    end: usize,
    buffer: &mut Buffer,
    register_handler: &mut RegisterHandler,
    mode: &mut Mode,
) {
    iterate_range(
        mode,
        register_handler,
        buffer,
        end,
        clear_reg,
        delete_char,
        noop,
    );
}

fn yank_char(register_handler: &mut RegisterHandler, buffer: &mut Buffer) {
    register_handler.push_reg(&buffer.get_curr_char().to_string());
    buffer.next_char();
}
pub fn yank(
    end: usize,
    buffer: &mut Buffer,
    register_handler: &mut RegisterHandler,
    mode: &mut Mode,
) {
    iterate_range(
        mode,
        register_handler,
        buffer,
        end,
        clear_reg,
        yank_char,
        reset_position,
    );
}

pub fn change(
    end: usize,
    buffer: &mut Buffer,
    register_handler: &mut RegisterHandler,
    mode: &mut Mode,
) {
    iterate_range(
        mode,
        register_handler,
        buffer,
        end,
        clear_reg,
        delete_char,
        insert,
    );
}
