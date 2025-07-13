use crate::Mode;

pub struct Command<F: FnOnce() -> ()> {
    mode: Mode,
    chain: Vec<char>,
    callback: F,
}

impl<F: FnOnce() -> ()> Command<F> {
    pub fn visual(chain: impl Into<Vec<char>>, callback: F) -> Self {
        Command {
            mode: Mode::Visual,
            chain: chain.into(),
            callback,
        }
    }

    pub fn normal(chain: &str, callback: F) -> Self {
        Command {
            mode: Mode::Normal,
            chain: chain.chars().collect(),
            callback,
        }
    }
}
