use anyhow::Result;
use std::sync::OnceLock;

#[macro_use]
mod utility;
mod action;
mod buffer;
mod buffer_char;
mod buffer_line;
mod buffer_update;
mod commands;
mod file_io;
mod highlight_c;
#[macro_use]
mod io;
mod meta_command;
mod mode;
mod motion;
mod operator;
mod panic_hook;
mod program_initialization;
mod program_loop;
mod register;
mod status_bar;
mod text_object;
mod tutorial;
mod undo;
mod view;
mod view_box;
mod view_command;

pub static DEBUG: OnceLock<bool> = OnceLock::new();

pub fn main() -> Result<()> {
    program_initialization::start_program()
}
