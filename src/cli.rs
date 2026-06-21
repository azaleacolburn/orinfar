use crate::mode::Mode;
use anyhow::{Result, bail};
use clap::Parser;
use std::path::PathBuf;

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
    pub fn parse_path() -> Result<(Self, Option<PathBuf>)> {
        let cli = Self::parse();

        match cli.file_name {
            Some(ref path) => {
                let path = PathBuf::from(path);
                let s = &path;

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
