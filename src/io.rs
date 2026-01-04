use anyhow::{Result, bail};
use ropey::Rope;
use std::{env, fs::OpenOptions, io::Write, path::PathBuf};

use clap::Parser;

use crate::{buffer::Buffer, mode::Mode};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    pub file_name: Option<String>,

    #[arg(short, long, value_enum, default_value_t = Mode::Normal)]
    pub mode: Mode,
    #[arg(short, long, default_value_t = true)]
    pub debug: bool,
}

impl Cli {
    pub fn parse_path() -> Result<(Cli, Option<PathBuf>)> {
        let cli = Self::parse();

        match cli.file_name {
            Some(ref path) => {
                let path = PathBuf::from(path);

                if path.is_dir() {
                    // TODO netrw
                    bail!("Orinfar does not support directory navigation");
                } else if path.is_file() {
                    Ok((cli, Some(path)))
                } else {
                    std::fs::write(&path, "")?;

                    Ok((cli, Some(path)))
                }
            }
            None => Ok((cli, None)),
        }
    }
}

pub fn load_file(path: &Option<PathBuf>, buffer: &mut Buffer) -> Result<()> {
    if let Some(path) = path {
        if !std::fs::exists(path)? {
            std::fs::write(path, buffer.rope.to_string())?;
            return Ok(());
        }

        let contents = std::fs::read_to_string(path)?;
        buffer.rope = Rope::from(contents);

        buffer.lines_for_updating = (0..buffer.len()).map(|_| true).collect::<Vec<bool>>();
        buffer.cursor = usize::min(buffer.cursor, buffer.rope.len_chars());
        buffer.has_changed = true;
    }

    Ok(())
}

pub fn write(path: PathBuf, buffer: Buffer) -> Result<()> {
    std::fs::write(path, buffer.to_string())?;

    Ok(())
}

pub fn log_dir() -> PathBuf {
    env::home_dir()
        .expect("Failed to get home dir")
        .join(".orinfar")
}

pub fn log_file() -> PathBuf {
    env::home_dir()
        .expect("Failed to get home dir")
        .join(".orinfar/log")
}

pub fn log(contents: impl ToString) {
    let mut file = OpenOptions::new()
        .append(true)
        .open(log_file())
        .expect("unable to open file");

    // append data to the file
    file.write_all(format!("{}\n", contents.to_string()).as_bytes())
        .expect("unable to append data");
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        if unsafe { DEBUG } {
            log(format!($($arg)*))
        }
    };
}
