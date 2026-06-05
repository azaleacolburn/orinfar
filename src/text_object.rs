use crate::{DEBUG, buffer::Buffer};

pub enum TextObjectType {
    Inside,
    Around,
}

pub type TOBounds = Option<(usize, usize)>;

pub struct TextObject<'a> {
    pub name: &'a str,
    command: fn(buffer: &Buffer) -> TOBounds,
}

// TODO
// Figure out if/how the `next_corresponding_bracket` code can be reused for text objects
// since I'm pretty sure if we just store a static map
// it'll handle half of our text objects (the ones with delimiting characters)
//
// Of course, other text objects are more complex.
//
// However, maybe motions passed-in to create text-objects arbitrarily and automatically
// e.g. the `word` text object is just `back` + `word`
//
// This is cool, but I think the best course of action is the boring one:
// 1. Move more (if not all) motion logic into the `Buffer` struct
// 2. Utilize the shared logic in both the sets of motion and text object functions
impl<'a> TextObject<'a> {
    pub fn new(name: &'a str, command: fn(buffer: &Buffer) -> TOBounds) -> Self {
        TextObject { name, command }
    }

    /// Returns the range that the text object occupies
    pub fn around(&self, buffer: &Buffer) -> TOBounds {
        (self.command)(buffer)
    }

    pub fn inside(&self, buffer: &Buffer) -> TOBounds {
        let (i, j) = (self.command)(buffer)?;

        if i == j {
            return Some((i, j));
        }

        Some((i + 1, j - 1))
    }
}

/// Returns the positions of the matching characters `start` and `end`
///
/// ## Matching Priority:
/// 1. Pair that you are currently on the opening character of
/// 2. Pair with the opening behind you
/// 3. Pair that you are currently on the closing character of
/// 4. Pair with the opening in front of you
///
/// ## TODO
/// - Prioritize the pair you are inside of,
///   separate from the one that starts behind you
pub fn find_matching(buffer: &Buffer, start: char, end: char) -> TOBounds {
    if buffer.get_curr_char() == start {
        let second = buffer.find_next_on_line(end)?;

        Some((buffer.cursor, second))
    } else if let Some(first) = buffer.find_prev_on_line(start) {
        let second = buffer.find_next_on_line_from(end, first)?;

        Some((first, second))
    } else if buffer.get_curr_char() == end {
        let first = buffer.find_prev_on_line(start)?;

        Some((first, buffer.cursor))
    } else {
        let first = buffer.find_next_on_line(start)?;
        let second = buffer.find_next_on_line_from(end, first)?;

        Some((first, second))
    }
}

pub fn parentheses(buffer: &Buffer) -> TOBounds {
    find_matching(buffer, '(', ')')
}

pub fn curly_braces(buffer: &Buffer) -> TOBounds {
    find_matching(buffer, '{', '}')
}

pub fn square_braces(buffer: &Buffer) -> TOBounds {
    find_matching(buffer, '[', ']')
}

pub fn quotations(buffer: &Buffer) -> TOBounds {
    find_matching(buffer, '\"', '\"')
}

pub fn single_quotations(buffer: &Buffer) -> TOBounds {
    find_matching(buffer, '\'', '\'')
}

pub fn grav(buffer: &Buffer) -> TOBounds {
    find_matching(buffer, '`', '`')
}
