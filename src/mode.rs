use std::io::stdout;

use crossterm::{cursor::SetCursorStyle, execute};

#[derive(Clone, Debug)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Visual,
}

impl Mode {
    pub fn insert(&mut self) {
        *self = Mode::Insert;
        execute!(stdout(), SetCursorStyle::BlinkingBar).unwrap();
    }

    pub fn normal(&mut self) {
        *self = Mode::Normal;
        execute!(stdout(), SetCursorStyle::SteadyBlock).unwrap();
    }
}
