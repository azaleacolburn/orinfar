use crate::{buffer::Buffer, view_box::ViewBox};
use anyhow::Result;
use crossterm::event::{Event, KeyCode, read};

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
    let symbols = "$`\'\":;~()\\+-=$#^[&]*<@%!{|}>/?.,";
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
pub fn on_next_input_view_box(
    view_box: &mut ViewBox,
    callback: fn(KeyCode, &mut ViewBox),
) -> Result<()> {
    loop {
        if let Event::Key(event) = read()? {
            callback(event.code, view_box);
            break;
        }
    }

    Ok(())
}

pub fn last_char(str: &str) -> char {
    str.chars().last().unwrap()
}
