use crate::{
    DEBUG, buffer::Buffer, io::try_get_git_hash, log, status_bar::StatusBar, undo::UndoTree,
};
use anyhow::Result;
use ropey::Rope;
use std::path::PathBuf;

pub fn substitute_cmd(
    buffer: &mut Buffer,
    status_bar: &StatusBar,
    undo_tree: &mut UndoTree,
    i: usize,
) {
    if status_bar[i..].len() == 1 {
        return;
    }
    let substitution: Vec<&[char]> = status_bar[i + 1..].split(|c| *c == '/').collect();

    if substitution.len() != 3 || !substitution[0].is_empty() {
        log!(
            "Malformed substitution meta-command: {:?}. Should be in the form: s/[orig]/[new]",
            substitution
        );

        return;
    }

    let original = substitution[1];
    let new: String = substitution[2].iter().collect();

    log!("Substition\n\toriginal: {:?}\n\tnew: {}", original, new);

    let idxs_of_substitution = buffer.find_occurences(original);
    log!("idxs of sub: {:?}", idxs_of_substitution);

    buffer.replace_text(
        &new,
        &original.iter().collect::<String>(),
        &idxs_of_substitution,
        undo_tree,
        false,
    );

    buffer.update_list_set(.., true);
    buffer.has_changed = true;
}

pub fn print_directories(
    buffer: &mut Buffer,
    undo_tree: &mut UndoTree,
    path: Option<PathBuf>,
) -> Result<()> {
    let path = path.map_or_else(
        || PathBuf::from("./"),
        |mut p| {
            p.pop();
            p
        },
    );

    let dir = std::fs::read_dir(path)?;
    let contents = dir
        .filter_map(std::result::Result::ok)
        .map(|item| {
            let mut path = item.file_name().to_string_lossy().to_string();
            path.push('\n');
            path
        })
        .collect::<String>();

    buffer.replace_contents(contents, undo_tree);

    Ok(())
}

pub fn attach_buffer(
    buffer: &mut Buffer,
    status_bar: &StatusBar,
    i: usize,
    path: &mut Option<PathBuf>,
    git_hash: &mut Option<String>,
) {
    if status_bar.len() == i + 1 {
        return;
    }
    let path_buf = PathBuf::from(status_bar[i + 1..].iter().collect::<String>().trim());
    log!("Set path to equal: {}", path_buf.to_string_lossy());
    // If we already have a file, we don't want to write the contents
    // to a new empty file
    if let Some(_path) = path {
        buffer.rope = Rope::new();
        buffer.cursor = 0;
        buffer.lines_for_updating = Vec::new();
        buffer.has_changed = true;
    }
    *path = Some(path_buf);

    *git_hash = try_get_git_hash(path.as_ref());
}
