use crate::buffer::Buffer;
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
    "$`\'\":;~()\\+-=$#^[&]*<@%!{|}>/?.,".contains(c)
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

pub fn last_char(str: &str) -> char {
    str.chars().last().unwrap()
}

pub const fn count_line(str: &str) -> u16 {
    let bytes = str.as_bytes();
    let mut i = 0;
    let mut len_lines = 0;
    // For loops aren't supported in `const` blocks yet
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            len_lines += 1;
        }
        i += 1;
    }

    len_lines
}

pub const fn count_longest_line(str: &str) -> u16 {
    let mut longest_line = 0;
    let mut curr_line = 0;
    // For loops aren't supported in `const` blocks yet
    let bytes = str.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            if curr_line > longest_line {
                longest_line = curr_line;
            }
            curr_line = 0;
        } else {
            curr_line += 1;
        }

        i += 1;
    }

    longest_line
}
