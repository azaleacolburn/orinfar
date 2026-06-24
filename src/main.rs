use anyhow::Result;
use std::sync::OnceLock;

#[macro_use]
mod utility;
mod action;
mod buffer;
mod buffer_char;
mod buffer_line;
mod buffer_update;
mod c;
mod cli;
mod commands;
mod count;
mod file_io;
mod global_state;
mod highlight;
#[macro_use]
mod logging;
mod language;
mod markdown;
mod meta_command;
mod mode;
mod motion;
mod operator;
mod panic_hook;
mod program_init;
mod program_loop;
mod quickfix;
mod register;
mod render;
mod status_bar;
mod text_object;
mod tutorial;
mod undo;
mod view;
mod view_box;
mod view_command;

pub static DEBUG: OnceLock<bool> = OnceLock::new();

pub fn main() -> Result<()> {
    program_init::start_program()
}
