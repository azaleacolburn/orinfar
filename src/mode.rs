use std::{fmt::Display, io::stdout};

use clap::ValueEnum;
use crossterm::{cursor::SetCursorStyle, execute};

#[derive(Clone, Debug, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum Mode {
    Normal,
    Insert,
    Meta,
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

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Mode::Normal => "normal",
            Mode::Insert => "insert",
            Mode::Meta => "meta",
            Mode::Visual => "visual",
        };

        f.write_str(str)
    }
}
