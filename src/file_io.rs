use std::path::PathBuf;

use crate::{DEBUG, io::log, log, view::View};
use anyhow::Result;
use ropey::Rope;
use tree_sitter::Parser;

impl View {
    pub fn load_file(&mut self) -> Result<()> {
        if let Some(path) = self.get_path().cloned() {
            let buffer = self.get_buffer_mut();
            if !std::fs::exists(&path)? {
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

    pub fn write(&self) -> Result<()> {
        let buffer = self.get_buffer().to_string();
        match self.get_path() {
            Some(path) => std::fs::write(path, buffer)?,
            None => log!("WARNING: Cannot Write Unattached Buffer"),
        }

        Ok(())
    }

    pub fn adjust(&mut self) -> bool {
        let view_box = self.get_view_box();
        view_box.adjust()
    }

    pub fn set_path(&mut self, path: Option<PathBuf>) {
        let view_box = &mut self.boxes[self.cursor];

        if let Some(path) = &path
            && let Some(ext) = path.extension()
            && (ext == "c" || ext == "h")
        {
            let mut parser = Parser::new();
            parser
                .set_language(&tree_sitter_c::LANGUAGE.into())
                .expect("Failed to load C parser");

            view_box.parser = Some(parser);
        }

        let git_hash = try_get_git_hash(path.as_ref());
        view_box.git_hash = git_hash;

        view_box.path = path;
    }

    pub fn get_path(&self) -> Option<&PathBuf> {
        let view_box = &self.boxes[self.cursor];

        view_box.path.as_ref()
    }

    pub fn get_git_hash(&self) -> Option<&str> {
        let view_box = &self.boxes[self.cursor];

        view_box.git_hash.as_deref()
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
            let head = head_str.split(' ').nth(1).unwrap();

            let ref_path = format!("{git_stem}/{head}");
            git_hash = std::fs::read_to_string(ref_path)
                .ok()
                .map(|s| s.trim().chars().take(7).collect::<String>());
        }
    }

    git_hash
}
