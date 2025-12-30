use std::io::stdout;

use clap::ValueEnum;
use crossterm::{cursor::SetCursorStyle, execute};

#[derive(Clone, Debug, ValueEnum)]
#[clap(rename_all = "kebab-case")]
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

impl ToString for Mode {
    fn to_string(&self) -> String {
        match self {
            Mode::Normal => "normal",
            Mode::Insert => "insert",
            Mode::Command => "command",
            Mode::Visual => "visual",
        }
        .to_string()
    }
}
