use anyhow::{Result, bail};
use ropey::Rope;
use std::{fs::File, io::BufReader, path::PathBuf};

use clap::Parser;

use crate::{buffer::Buffer, log, status_bar::StatusBar, view_box::ViewBox};

#[derive(Parser)]
pub struct Cli {
    pub file_name: Option<String>,
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
