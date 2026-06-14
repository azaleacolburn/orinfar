use crate::{
    DEBUG, buffer::Buffer, file_io::try_get_git_hash, global_state::GlobalState, mode::Mode,
    undo::UndoTree, utility::SplitOnce, view::View, view_box::ViewBox,
    view_command::split_curr_view_box_horizontal,
};
use anyhow::Result;
use ropey::Rope;
use std::path::PathBuf;
use tree_sitter::Parser;

// TODO
// Eventually match from a list of `MatchCommand`s to make them easier to manage
// (this is fine for now though)

/// # Returns
/// A boolean indicating whether to break from the main program loop
pub fn match_meta_command(global_state: &mut GlobalState, view: &mut View) -> Result<bool> {
    let (command, arg): (&[char], &[char]) = global_state.status_bar[1..]
        .split_once_a(|c| *c == ' ' || *c == '/')
        .unwrap_or_else(|| (&global_state.status_bar[1..], &[]));
    let (command, arg): (String, String) = (command.iter().collect(), arg.iter().collect());

    match command.as_str() {
        "write" | "w" => view.write()?,
        "quit" | "q" => return Ok(true),
        "wq" => {
            view.write()?;
            return Ok(true);
        }

        "unattach" | "u" => view.set_path(None),

        "load" | "l" => {
            view.load_file()?;
            let view_box = view.get_view_box();
            view_box.flush(false)?;
        }

        "open" | "o" => {
            attach_buffer(&arg, view.get_view_box());
            view.load_file()?;

            let view_box = view.get_view_box();
            view_box.flush(false)?;
        }

        "sub" | "s" => {
            let buffer = view.get_buffer_mut();
            substitute_cmd(buffer, &arg, &mut global_state.undo_tree);
        }

        "dir" | "d" => {
            if view.get_buffer().rope.len_chars() == 0 {
                print_directories(view, &mut global_state.undo_tree)?;
                return Ok(false);
            }

            split_curr_view_box_horizontal(view);

            let anchor = view.cursor;
            view.cursor = view.boxes.len() - 1;

            print_directories(view, &mut global_state.undo_tree)?;
            view.cursor = anchor;
        }

        "reg" => {
            let registers = &mut global_state.register_handler.to_string();

            if view.get_buffer().rope.len_chars() == 0 {
                view.get_buffer_mut()
                    .replace_contents(registers, &mut global_state.undo_tree);
            }

            split_curr_view_box_horizontal(view);

            let anchor = view.cursor;
            view.cursor = view.boxes.len() - 1;

            view.get_buffer_mut()
                .replace_contents(registers, &mut global_state.undo_tree);

            view.cursor = anchor;
        }

        n => {
            if let Ok(num) = n.parse::<usize>() {
                let buffer = view.get_buffer_mut();
                buffer.set_row(num + 1);
            } else {
                log!("Unknown Meta-Command: {}", n);
            }
        }
    }

    global_state.mode = Mode::Normal;
    global_state.status_bar.clear();

    Ok(false)
}

pub fn substitute_cmd(buffer: &mut Buffer, arg: &str, undo_tree: &mut UndoTree) {
    if arg.len() < 3 {
        return;
    }

    let substitution: Vec<&str> = arg.split('/').collect();

    if substitution.len() != 2 {
        log!(
            "Malformed substitution meta-command: {:?}. Should be in the form: s/[orig]/[new]",
            substitution
        );

        return;
    }

    let original: Vec<char> = substitution[0].chars().collect();
    let new: String = substitution[1].chars().collect();

    let idxs_of_substitution = buffer.find_occurences(&original);
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
    let path = PathBuf::from("./");

    let dir = std::fs::read_dir(path)?;
    let contents = dir
        .filter_map(std::result::Result::ok)
        .map(|item| {
            let mut path = item.file_name().to_string_lossy().to_string();
            path.push('\n');
            path
        })
        .collect::<String>();

    // Some issue with replacing contents that has a trailing newline
    view.get_buffer_mut()
        .replace_contents(&contents[0..contents.len() - 1], undo_tree);

    Ok(())
}

pub fn attach_buffer(arg: &str, view_box: &mut ViewBox) {
    let path_buf = PathBuf::from(arg.trim());

    // If we already have a file, we don't want to write the contents
    // to a new empty file
    if let Some(_path) = &view_box.path() {
        view_box.buffer.rope = Rope::new();
        view_box.buffer.cursor = 0;
        view_box.buffer.lines_for_updating = Vec::new();
        view_box.buffer.has_changed = true;
    }
    view_box.set_path(Some(path_buf.clone()));
    view_box.git_hash = try_get_git_hash(view_box.path());

    if let Some(ext) = path_buf.extension()
        && (ext == "c" || ext == "h")
    {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_c::LANGUAGE.into())
            .expect("Failed to load C parser");

        view_box.parser = Some(parser);
    }
}
