use anyhow::{Result, bail};
use ropey::Rope;
use std::{env, fs::OpenOptions, io::Write, path::PathBuf};

use clap::Parser;

use crate::{buffer::Buffer, mode::Mode, view::View};

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

pub fn try_get_git_hash(path: Option<&PathBuf>) -> Option<String> {
    let mut git_hash: Option<String> = None;
    if let Some(path) = path {
        let path = if path.is_dir() {
            path
        } else {
            path.parent().unwrap()
        }
        .to_str()
        .unwrap();

        let git_stem = if path.is_empty() {
            String::from(".git")
        } else {
            format!("{path}/.git")
        };

        let head_path = format!("{git_stem}/HEAD");

        if let Ok(head_str) = std::fs::read_to_string(head_path).map(|s| s.trim().to_string()) {
            let head = head_str.split(' ').collect::<Vec<&str>>()[1];

            let ref_path = format!("{git_stem}/{head}");
            git_hash = std::fs::read_to_string(ref_path)
                .ok()
                .map(|s| s.trim().chars().take(7).collect::<String>());
        }
    }

    git_hash
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

pub fn log(contents: &impl ToString) {
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
            log(&format!($($arg)*))
        }
    };
}
