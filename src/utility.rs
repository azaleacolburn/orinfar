use crate::buffer::Buffer;
use anyhow::Result;
use crossterm::event::{Event, KeyCode, read};
use std::ops::RangeBounds;

macro_rules! unwrap_or_return {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None => return,
        }
    };
}

macro_rules! unwrap_or_break {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None => break,
        }
    };
}

pub fn is_symbol(c: char) -> bool {
    let symbols = "$`\'\":~()\\+-=$#^[&]*<@%!{|}>/?.,";
    symbols.contains(c)
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
pub fn on_next_input(buffer: &mut Buffer, callback: fn(KeyCode, &mut Buffer)) -> Result<()> {
    loop {
        if let Event::Key(event) = read()? {
            callback(event.code, buffer);
            break;
        }
    }

    Ok(())
}
