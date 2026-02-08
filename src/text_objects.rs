use crate::buffer::Buffer;

pub struct TextObject<'a> {
    pub name: &'a str,
    command: fn(buffer: &Buffer) -> (usize, usize),
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
    pub fn new(name: &'a str, command: fn(buffer: &Buffer) -> (usize, usize)) -> Self {
        TextObject { name, command }
    }

    // Returns the range that the text object occupies
    pub fn evaluate(&self, buffer: &Buffer) -> (usize, usize) {
        (self.command)(&buffer)
    }
}

fn parentheses(buffer: &Buffer) -> (usize, usize) {}
