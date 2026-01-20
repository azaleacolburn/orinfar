use crate::{
    DEBUG, buffer::Buffer, io::try_get_git_hash, log, mode::Mode, register::RegisterHandler,
    status_bar::StatusBar, undo::UndoTree, view::View, view_box::ViewBox,
    view_command::split_curr_view_box_vertical,
};
use anyhow::Result;
use ropey::Rope;
use std::path::PathBuf;

#[allow(clippy::too_many_arguments)]
/// # Returns
/// A boolean indicating whether to break from the main program loop
pub fn match_meta_command(
    status_bar: &mut StatusBar,
    view: &mut View,
    register_handler: &RegisterHandler,
    undo_tree: &mut UndoTree,
    mode: &mut Mode,
) -> Result<bool> {
    for (i, command) in status_bar.iter().enumerate().skip(1) {
        match command {
            'w' => view.write()?,
            'u' => {
                view.set_path(None);
            }
            'l' => {
                view.load_file()?;
                let view_box = view.get_view_box();
                view_box.flush(false)?;
            }
            'o' => {
                attach_buffer(status_bar, i, view.get_view_box());
                view.load_file()?;

                let view_box = view.get_view_box();
                view_box.flush(false)?;
                break;
            }
            'd' => {
                split_curr_view_box_vertical(view);
                print_directories(view, undo_tree)?;
            }
            // Print Registers
            'r' => {
                let registers = register_handler.to_string();
                view.split_view_box_vertical(view.cursor);

                let last_idx = view.boxes_len() - 1;
                view.replace_buffer_contents(&registers, last_idx, undo_tree);
            }

            's' => {
                let buffer = view.get_buffer_mut();
                substitute_cmd(buffer, status_bar, undo_tree, i);
                break;
            }
            n if n.is_numeric() => {
                let num_str = status_bar[i..].iter().collect::<String>();
                let num: usize = match num_str.parse() {
                    Ok(n) => n,
                    Err(err) => {
                        log!("Failed to parse number: {} ({})", num_str, err);
                        break;
                    }
                };

                let buffer = view.get_buffer_mut();
                buffer.set_row(num + 1);
            }
            'q' => return Ok(true),
            c => log!("Unknown Meta-Command: {}", c),
        }
    }

    *mode = Mode::Normal;
    status_bar.clear();

    Ok(false)
}

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

pub fn print_directories(view: &mut View, undo_tree: &mut UndoTree) -> Result<()> {
    let path = view.get_path().map_or_else(
        || PathBuf::from("./"),
        |p| {
            let mut p = p.clone();
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

    view.get_buffer_mut().replace_contents(contents, undo_tree);

    Ok(())
}

pub fn attach_buffer(status_bar: &StatusBar, i: usize, view_box: &mut ViewBox) {
    if status_bar.len() == i + 1 {
        return;
    }
    let path_buf = PathBuf::from(status_bar[i + 1..].iter().collect::<String>().trim());
    log!("Set path to equal: {}", path_buf.to_string_lossy());
    // If we already have a file, we don't want to write the contents
    // to a new empty file
    if let Some(_path) = &view_box.path {
        view_box.buffer.rope = Rope::new();
        view_box.buffer.cursor = 0;
        view_box.buffer.lines_for_updating = Vec::new();
        view_box.buffer.has_changed = true;
    }
    view_box.path = Some(path_buf);
    view_box.git_hash = try_get_git_hash(view_box.path.as_ref());
}
