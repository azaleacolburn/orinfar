use anyhow::{bail, Result};
use std::path::PathBuf;

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
                let contents = std::fs::read_to_string(path.clone())?;
                buffer.buff = vec![];
                contents
                    .split('\n')
                    .for_each(|line| buffer.push_line(line.chars().collect::<Vec<char>>()));
                buffer.flush();
            }
            Some(path)
        }
        None => None,
    })
}

pub fn write(path: PathBuf, buffer: Buffer) -> Result<()> {
    // There's probably a more efficient way of doing this
    let buf = buffer
        .buff
        .into_iter()
        .map(|line| line.into_iter().chain(std::iter::once('\n')))
        .flatten()
        .collect::<String>();
    std::fs::write(path, buf)?;

    Ok(())
}
