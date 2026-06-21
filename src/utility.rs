use std::any::Any;

use crate::{DEBUG, buffer::Buffer, log, logn, view_command::move_up_one_view_box};
use anyhow::Result;
use crossterm::event::{Event, KeyCode, read};
use tree_sitter::{Tree, TreeCursor};

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

pub fn traverse_tree<T: Default>(
    tree: &Tree,
    mut node_action: impl FnMut(&TreeCursor, &mut T),
    mut move_up_hook: impl FnMut(&mut T),
    mut move_down_hook: impl FnMut(&mut T),
) -> T {
    let mut cursor = tree.walk();
    let mut state = T::default();

    'TREE_WALK: loop {
        node_action(&cursor, &mut state);

        if cursor.goto_first_child() {
            move_down_hook(&mut state);
            continue;
        }

        if cursor.goto_next_sibling() {
            continue;
        }

        'BACKTRACKING: loop {
            if !cursor.goto_parent() {
                break 'TREE_WALK;
            } else {
                move_up_hook(&mut state);
            }

            if cursor.goto_next_sibling() {
                break 'BACKTRACKING;
            }
        }
    }

    state
}

pub fn print_tree(tree: &Tree) {
    let node_action = |cursor: &TreeCursor<'_>, depth: &mut usize| {
        let node = cursor.node();
        for _ in 0..*depth {
            logn!("\t");
        }
        let kind = node.kind();
        let start = node.start_position();
        let end = node.end_position();
        log!(
            "{} [{}, {}] - [{}, {}]",
            kind,
            start.row,
            start.column,
            end.row,
            end.column
        );
    };

    let move_up_hook = |depth: &mut usize| {
        *depth -= 1;
    };
    let move_down_hook = |depth: &mut usize| {
        *depth += 1;
    };

    traverse_tree(tree, node_action, move_up_hook, move_down_hook);
}
