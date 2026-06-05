use crate::{
    DEBUG,
    action::enumerate_normal_chars,
    buffer::Buffer,
    cli::Cli,
    commands::{
        Command as Cmd, append, cut, first_row, insert, insert_new_line, insert_new_line_above,
        last_row, paste, replace, set_curr_register, undo,
    },
    logging::{data_file, log_dir, log_file, write_data},
    mode::Mode,
    motion::Motion,
    operator::{Operator, change, delete, yank},
    panic_hook,
    program_loop::program_loop,
    register::RegisterHandler,
    status_bar::StatusBar,
    text_object::{
        TextObject, TextObjectType, curly_braces, grav, parentheses, quotations, single_quotations,
        square_braces,
    },
    undo::UndoTree,
    view::{View, cleanup, terminal_setup},
    view_command::{
        ViewCommand, center_viewbox_on_cursor, delete_curr_view_box, move_down_one_view_box,
        move_left_one_view_box, move_right_one_view_box, move_up_one_view_box,
        split_curr_view_box_horizontal, split_curr_view_box_vertical,
    },
};
use anyhow::{Result, bail};
use crossterm::terminal::size;

#[allow(clippy::too_many_lines)]
pub fn start_program() -> Result<()> {
    let (cli, path) = Cli::parse_path()?;
    if let Err(_b) = DEBUG.set(cli.debug) {
        bail!("Failed to set DEBUG variable");
    }

    let (cols, rows) = size()?;
    terminal_setup(rows, cols)?;

    panic_hook::add_panic_hook(&cleanup);

    // This could fail if the dir already exists, so we don't care if this fails
    if let Err(err) = std::fs::create_dir(log_dir())
        && err.to_string() != "File exists (os error 17)"
    {
        return Err(err.into());
    }
    std::fs::File::create(log_file())?;
    let data_path = data_file();
    if !data_path.exists() {
        std::fs::File::create(&data_path)?;
    }

    let data = std::fs::read_to_string(&data_path)?;

    let mut has_opened = false;
    data.lines().for_each(|l| {
        let Some((k, v)) = l.split_once(':') else {
            panic!("Invalid Data File");
        };

        has_opened |= ("has_opened", "true") == (k.trim(), v.trim());
    });

    let undo_tree = UndoTree::new();
    let register_handler = RegisterHandler::new();
    let status_bar: StatusBar = StatusBar::new();

    let view_commands: &[ViewCommand] = &[
        ViewCommand::new("zz", center_viewbox_on_cursor),
        // View Box related
        ViewCommand::new("zd", move_down_one_view_box),
        ViewCommand::new("zu", move_up_one_view_box),
        ViewCommand::new("zl", move_left_one_view_box),
        ViewCommand::new("zr", move_right_one_view_box),
        ViewCommand::new("zx", delete_curr_view_box),
        ViewCommand::new("zv", split_curr_view_box_vertical),
        ViewCommand::new("zh", split_curr_view_box_horizontal),
    ];

    let commands: &[Cmd] = &[
        // Insert
        Cmd::new("i", insert),
        Cmd::new("a", append),
        Cmd::new("o", insert_new_line),
        Cmd::new("O", insert_new_line_above),
        // Single character edit
        Cmd::new("x", cut),
        Cmd::new("r", replace),
        // File Traversal
        Cmd::new("G", last_row),
        Cmd::new("gg", first_row),
        // Misc
        Cmd::new("u", undo),
        Cmd::new("p", paste),
        Cmd::new("\"", set_curr_register),
    ];

    let operators: &[Operator] = &[
        Operator::new("d", delete),
        Operator::new("y", yank),
        Operator::new("t", change),
    ];

    let motions: &[Motion] = &[
        // HJKL
        Motion::inclusive('h', Buffer::prev_char),
        Motion::inclusive('j', Buffer::next_row),
        Motion::inclusive('k', Buffer::prev_row),
        Motion::inclusive('l', Buffer::next_char),
        // Word operators
        Motion::exclusive('w', Buffer::word),
        Motion::exclusive('b', Buffer::back),
        Motion::inclusive('e', Buffer::end_of_word),
        // Line operators
        Motion::inclusive('$', Buffer::end_of_line),
        Motion::inclusive('_', Buffer::beginning_of_line),
        // Finding operators
        Motion::inclusive('f', Buffer::find),
        Motion::inclusive('F', Buffer::find_back),
        Motion::inclusive('c', Buffer::find_until),
        // Paragraph operators
        Motion::inclusive('%', Buffer::next_corresponding_bracket),
        Motion::inclusive('}', Buffer::next_newline),
        Motion::inclusive('{', Buffer::prev_newline),
    ];

    let text_objects: &[TextObject] = &[
        // Parentheses
        TextObject::new("(", parentheses),
        TextObject::new(")", parentheses),
        TextObject::new("p", parentheses),
        // Curly Braces
        TextObject::new("{", curly_braces),
        TextObject::new("}", curly_braces),
        TextObject::new("c", curly_braces),
        // Square Braces
        TextObject::new("[", square_braces),
        TextObject::new("]", square_braces),
        TextObject::new("s", square_braces),
        // Quatations
        TextObject::new("\"", quotations),
        TextObject::new("\'", single_quotations),
        TextObject::new("`", grav),
    ];

    let next_operation: Option<&Operator> = None;
    let text_object_type: Option<TextObjectType> = None;

    // Used for not putting excluded chars in the chain
    let all_normal_chars =
        enumerate_normal_chars(commands, operators, motions, text_objects, view_commands);

    let mut view: View = View::new(cols, rows);

    let mode = Mode::Normal;
    let count: u16 = 1;
    let chained: Vec<char> = vec![];
    let search_str: Vec<char> = vec![];

    if !has_opened && path.is_none() {
        view.get_view_box().write_welcome_screen();
        write_data(&"has_opened", &"true");
    }

    view.set_path(path);
    view.load_file()?;

    let _ = view.get_view_box().parse();

    view.flush(
        &status_bar,
        &mode,
        &chained,
        count,
        register_handler.get_curr_reg(),
        false,
    )?;

    program_loop(
        commands,
        operators,
        motions,
        text_objects,
        view_commands,
        count,
        chained,
        next_operation,
        text_object_type,
        &all_normal_chars,
        search_str,
        status_bar,
        register_handler,
        undo_tree,
        view,
        mode,
    )?;

    cleanup()
}
