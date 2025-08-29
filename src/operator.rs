use crate::{
    buffer::Buffer,
    motion::{end_of_line, Motion},
    register::RegisterHandler,
    Cursor, Mode,
};

pub struct Operator<'a> {
    pub name: &'a [char],
    command: fn(
        end: Cursor,
        buffer: &mut Buffer,
        register_handler: &mut RegisterHandler,
        mode: &mut Mode,
    ),
}

impl<'a> Operator<'a> {
    pub fn new(
        name: &'a [char],
        command: fn(
            end: Cursor,
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
        let orig_col = buffer.cursor.col;
        buffer.cursor.col = 0;
        let end_of_line = Cursor {
            row: buffer.row(),
            col: buffer.get_end_of_row(),
        };

        (self.command)(end_of_line, buffer, register_handler, mode);
        let len = buffer.get_curr_line().len();
        if len > 0 {
            buffer.cursor.col = usize::min(orig_col, len - 1);
        }
    }
}

pub fn iterate_range(
    mode: &mut Mode,
    register_handler: &mut RegisterHandler,
    buffer: &mut Buffer,
    end: Cursor,
    initial_callback: fn(
        register_handler: &mut RegisterHandler,
        buffer: &mut Buffer,
        mode: &mut Mode,
    ),
    iter_callback: fn(register_handler: &mut RegisterHandler, buffer: &mut Buffer),
    after_callback: fn(
        start: Cursor,
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
    _start: Cursor,
    _register_handler: &mut RegisterHandler,
    _buffer: &mut Buffer,
    _mode: &mut Mode,
) {
}
fn reset_position(
    start: Cursor,
    _register_handler: &mut RegisterHandler,
    buffer: &mut Buffer,
    _mode: &mut Mode,
) {
    buffer.cursor = start
}
fn insert(
    _start: Cursor,
    _register_handler: &mut RegisterHandler,
    _buffer: &mut Buffer,
    mode: &mut Mode,
) {
    mode.insert();
}

fn clear_reg(register_handler: &mut RegisterHandler, _buffer: &mut Buffer, _mode: &mut Mode) {
    register_handler.set_reg(Vec::new());
}

fn delete_char(register_handler: &mut RegisterHandler, buffer: &mut Buffer) {
    register_handler.push_char(buffer.get_curr_char());
    buffer.delete_curr_char();
}
pub fn delete(
    end: Cursor,
    buffer: &mut Buffer,
    register_handler: &mut RegisterHandler,
    mode: &mut Mode,
) {
    if buffer.get_curr_line().len() == buffer.cursor.col {
        return;
    }
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
    register_handler.push_char(buffer.get_curr_char());
    buffer.next_char();
}
pub fn yank(
    end: Cursor,
    buffer: &mut Buffer,
    register_handler: &mut RegisterHandler,
    mode: &mut Mode,
) {
    if buffer.get_curr_line().len() == buffer.cursor.col {
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
    );
}

pub fn change(
    end: Cursor,
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
