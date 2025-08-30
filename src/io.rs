use anyhow::{bail, Result};
use ropey::Rope;
use std::{fs::File, io::BufReader, path::PathBuf};

use clap::Parser;

use crate::buffer::Buffer;

#[derive(Parser)]
pub struct Cli {
    pub file_name: Option<String>,
}

pub fn load_file(buffer: &mut Buffer) -> Result<Option<PathBuf>> {
    let cli = Cli::parse();
    // TODO This is a bad way of handling things, refactor later
    Ok(match cli.file_name {
        Some(path) => {
            let path = PathBuf::from(path);

            if path.is_dir() {
                // TODO netrw
                bail!("Orinfar does not support directory navigation");
            } else if path.is_file() {
                buffer.rope = Rope::from_reader(BufReader::new(File::create_new(path)?))?;
                buffer.flush();
            }
            Some(path)
        }
        None => None,
    })
}

pub fn write(path: PathBuf, buffer: Buffer) -> Result<()> {
    std::fs::write(path, buffer.to_string())?;

    Ok(())
}
