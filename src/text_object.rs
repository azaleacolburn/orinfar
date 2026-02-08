use crate::{DEBUG, buffer::Buffer, log, motion::find_back};

pub enum TextObjectType {
    Inside,
    Around,
}

pub struct TextObject<'a> {
    pub name: &'a str,
    command: fn(buffer: &Buffer) -> Option<(usize, usize)>,
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
    pub fn new(name: &'a str, command: fn(buffer: &Buffer) -> Option<(usize, usize)>) -> Self {
        TextObject { name, command }
    }

    // Returns the range that the text object occupies
    pub fn around(&self, buffer: &Buffer) -> Option<(usize, usize)> {
        (self.command)(&buffer)
    }

    pub fn inside(&self, buffer: &Buffer) -> Option<(usize, usize)> {
        let Some((i, j)) = (self.command)(&buffer) else {
            return None;
        };

        if i == j {
            return Some((i, j));
        }

        Some((i + 1, j - 1))
    }
}

pub fn parentheses(buffer: &Buffer) -> Option<(usize, usize)> {
    log!("here");
    let Some(first) = buffer.find_prev('(') else {
        return None;
    };

    log!("first: {first}");

    let Some(second) = buffer.find_next(')') else {
        return None;
    };

    Some((first, second))
}
