use crate::{
    DEBUG,
    buffer::Buffer,
    io::{self, try_get_git_hash},
    log,
    mode::Mode,
    register::RegisterHandler,
    status_bar::StatusBar,
    undo::UndoTree,
    view_box::ViewBox,
};
use anyhow::Result;
use ropey::Rope;
use std::path::PathBuf;

#[allow(clippy::too_many_arguments)]
/// # Returns
/// A boolean indicating whether to break from the main program loop
pub fn match_meta_command(
    buffer: &mut Buffer,
    status_bar: &mut StatusBar,
    view_box: &ViewBox,
    register_handler: &RegisterHandler,
    undo_tree: &mut UndoTree,
    mode: &mut Mode,

    chained: &[char],
    count: u16,

    git_hash: &mut Option<String>,
    path: &mut Option<PathBuf>,
) -> Result<bool> {
    for (i, command) in status_bar.iter().enumerate().skip(1) {
        match command {
            'w' => match &path {
                Some(path) => {
                    io::write(path.clone(), buffer)?;
                }
                None => log!("WARNING: Cannot Write Unattached Buffer"),
            },
            'u' => {
                *path = None;
            }
            'l' => {
                io::load_file(path.as_ref(), buffer)?;
                view_box.flush(
                    buffer,
                    status_bar,
                    mode,
                    chained,
                    count,
                    register_handler.get_curr_reg(),
                    path.as_ref(),
                    git_hash.as_deref(),
                    false,
                )?;
            }
            'o' => {
                attach_buffer(buffer, status_bar, i, path, git_hash);

                io::load_file(path.as_ref(), buffer)?;
                view_box.flush(
                    buffer,
                    status_bar,
                    mode,
                    chained,
                    count,
                    register_handler.get_curr_reg(),
                    path.as_ref(),
                    git_hash.as_deref(),
                    false,
                )?;
                break;
            }
            'd' => {
                print_directories(buffer, undo_tree, path.clone())?;
            }
            // Print Registers
            'r' => {
                let registers = register_handler.to_string();
                buffer.replace_contents(registers, undo_tree);
            }
            's' => {
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
