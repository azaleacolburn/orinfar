use crate::buffer::Buffer;
use anyhow::Result;
use crossterm::event::{Event, KeyCode, read};

macro_rules! unwrap_or_return {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None => {
                return;
            }
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

/// Trys to convert the value `$e` into the type `$t`
/// Evaluates into `$e` of the new type `$t` on success
/// Returns from the current function on failure
macro_rules! try_into_or_return {
    ($t:ty, $e:expr) => {
        unwrap_or_return!(<$t>::try_from($e).ok())
    };
}

// TODO
// Remove extension trait when `split_once` becomes stable
// [Tracking Issue](https://github.com/rust-lang/rust/issues/112811)
pub trait SplitOnce<T> {
    fn split_once_a<F>(&self, pred: F) -> Option<(&[T], &[T])>
    where
        F: FnMut(&T) -> bool;
}

impl<T> SplitOnce<T> for [T] {
    #[inline]
    fn split_once_a<F>(&self, pred: F) -> Option<(&[T], &[T])>
    where
        F: FnMut(&T) -> bool,
    {
        let index = self.iter().position(pred)?;

        Some((&self[..index], &self[index + 1..]))
    }
}

pub fn is_symbol(c: char) -> bool {
    "$`\'\":;~()\\+-=$#^[&]*<@%!{|}>/?.,".contains(c)
}

/// # Errors
/// - I/O error if `crossterm::events::read()` fails
pub fn on_next_input(buffer: &mut Buffer, closure: fn(KeyCode, &mut Buffer)) -> Result<()> {
    loop {
        if let Event::Key(event) = read()? {
            closure(event.code, buffer);
            break;
        }
    }

    Ok(())
}

/// Returns `\0` if the string is empty
pub fn last_char(str: &str) -> char {
    str.chars().last().unwrap_or('\0')
}

pub const fn count_lines(str: &str) -> u16 {
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
