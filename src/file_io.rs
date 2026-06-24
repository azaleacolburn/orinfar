use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{DEBUG, log, view::View};
use anyhow::Result;
use ropey::Rope;

impl View {
    pub fn load_file(&mut self) -> Result<()> {
        let Some(path) = self.get_path().cloned() else {
            return Ok(());
        };

        let buffer = self.get_buffer_mut();
        if !fs::exists(&path)? {
            fs::write(path, buffer.rope.to_string())?;
            return Ok(());
        }

        let contents = fs::read_to_string(path)?;
        buffer.rope = Rope::from(contents);

        buffer.lines_for_updating = (0..buffer.len()).map(|_| true).collect::<Vec<bool>>();
        buffer.cursor = usize::min(buffer.cursor, buffer.rope.len_chars());
        buffer.has_changed = true;

        Ok(())
    }

    pub fn write(&self) -> Result<()> {
        match self.get_path() {
            Some(path) => {
                let buffer = self.get_buffer().to_string();
                fs::write(path, buffer)?;
            }

            None => log!("WARNING: Cannot Write Unattached Buffer"),
        }

        Ok(())
    }

    pub fn adjust(&mut self) -> bool {
        let view_box = self.get_view_box_mut();
        view_box.adjust()
    }

    pub fn set_path(&mut self, path: Option<PathBuf>) {
        let view_box = self.get_view_box_mut();

        view_box.set_path(path);
    }

    pub fn get_path(&self) -> Option<&PathBuf> {
        let view_box = self.get_view_box();

        view_box.path()
    }

    pub fn get_git_hash(&self) -> Option<&str> {
        let view_box = self.get_view_box();

        view_box.git_hash.as_deref()
    }
}

pub fn try_get_git_hash(path: Option<&PathBuf>) -> Option<String> {
    let mut git_hash: Option<String> = None;

    if let Some(path) = path {
        let path = if path.is_dir() {
            path
        } else {
            // NOTE
            // I believe the only time this will fail is in the root directory
            path.parent().unwrap_or_else(|| Path::new("/"))
        }
        .to_str()
        .unwrap_or("");

        let git_stem = if path.is_empty() {
            String::from(".git")
        } else {
            format!("{path}/.git")
        };

        let head_path = format!("{git_stem}/HEAD");

        if let Ok(head_str) = std::fs::read_to_string(head_path).map(|s| s.trim().to_string())
            && let Some(head) = head_str.split(' ').nth(1)
        {
            let ref_path = format!("{git_stem}/{head}");
            git_hash = std::fs::read_to_string(ref_path)
                .ok()
                .map(|s| s.trim().chars().take(7).collect::<String>());
        }
    }

    git_hash
}
