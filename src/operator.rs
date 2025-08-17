use crate::{
    buffer::{self, Buffer},
    motion::Motion,
    register::{self, RegisterHandler},
    Cursor, Mode,
};

pub struct Operator {
    pub name: Vec<char>,
    command: fn(
        end: Cursor,
        buffer: &mut Buffer,
        register_handler: &mut RegisterHandler,
        mode: &mut Mode,
    ),
}

impl Operator {
    pub fn new(
        name: &str,
        command: fn(
            end: Cursor,
            buffer: &mut Buffer,
            register_handler: &mut RegisterHandler,
            mode: &mut Mode,
        ),
    ) -> Self {
        Self {
            name: name.chars().collect(),
            command,
        }
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
}

pub fn iterate_range(
    register_handler: &mut RegisterHandler,
    buffer: &mut Buffer,
    end: Cursor,
    initial_callback: fn(register_handler: &mut RegisterHandler, buffer: &mut Buffer),
    iter_callback: fn(register_handler: &mut RegisterHandler, buffer: &mut Buffer),
    after_callback: fn(start: Cursor, register_handler: &mut RegisterHandler, buffer: &mut Buffer),
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

    initial_callback(register_handler, buffer);
    (0..=count)
        .into_iter()
        .for_each(|_| iter_callback(register_handler, buffer));
    after_callback(anchor_cursor, register_handler, buffer);
}

fn noop(_start: Cursor, _register_handler: &mut RegisterHandler, _buffer: &mut Buffer) {}
fn reset_position(start: Cursor, _register_handler: &mut RegisterHandler, buffer: &mut Buffer) {
    buffer.cursor = start
}

fn clear_reg(register_handler: &mut RegisterHandler, _buffer: &mut Buffer) {
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
    _mode: &mut Mode,
) {
    iterate_range(register_handler, buffer, end, clear_reg, delete_char, noop);
}

fn yank_char(register_handler: &mut RegisterHandler, buffer: &mut Buffer) {
    register_handler.push_char(buffer.get_curr_char());
    buffer.next_char();
}
pub fn yank(
    end: Cursor,
    buffer: &mut Buffer,
    register_handler: &mut RegisterHandler,
    _mode: &mut Mode,
) {
    iterate_range(
        register_handler,
        buffer,
        end,
        clear_reg,
        yank_char,
        reset_position,
    );
}

fn change() {}
