use std::{fmt::Display, io::stdout};

use clap::ValueEnum;
use crossterm::{cursor::SetCursorStyle, execute};

#[derive(Clone, Debug, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum Mode {
    Normal,
    Insert,
    Meta,
    Search,
    Visual,
}

impl Mode {
    pub fn insert(&mut self) {
        *self = Self::Insert;
        execute!(stdout(), SetCursorStyle::BlinkingBar)
            .expect("Crossterm blinking bar command failed");
    }

    pub fn normal(&mut self) {
        *self = Self::Normal;
        execute!(stdout(), SetCursorStyle::SteadyBlock)
            .expect("Crossterm steady block command failed");
    }

    pub fn search(&mut self) {
        *self = Self::Search;
        execute!(stdout(), SetCursorStyle::SteadyBlock)
            .expect("Crossterm steady block command failed");
    }
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Normal => "normal",
            Self::Insert => "insert",
            Self::Meta => "meta",
            Self::Search => "search",
            Self::Visual => "visual",
        };

        f.write_str(str)
    }
}
