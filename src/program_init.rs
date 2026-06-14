use crate::{
    DEBUG,
    action::enumerate_normal_chars,
    buffer::Buffer,
    cli::Cli,
    commands::{
        Command as Cmd, append, cut, first_row, indent, insert, insert_new_line,
        insert_new_line_above, last_row, paste, replace, set_curr_register, undo, unindent,
    },
    global_state::GlobalState,
    logging::{setup_logging_and_data, write_data},
    motion::Motion,
    operator::{Operator, change, delete, yank},
    panic_hook,
    program_loop::program_loop,
    text_object::{
        TextObject, curly_braces, grav, parentheses, quotations, single_quotations, square_braces,
    },
    view::{View, cleanup, terminal_setup},
    view_command::{
        ViewCommand, center_viewbox_on_cursor, delete_curr_view_box, move_down_one_view_box,
        move_left_one_view_box, move_right_one_view_box, move_up_one_view_box,
        split_curr_view_box_horizontal, split_curr_view_box_vertical,
    },
};
use anyhow::{Result, bail};
use crossterm::terminal::size;

pub fn start_program() -> Result<()> {
    let (cli, path) = Cli::parse_path()?;
    if let Err(_b) = DEBUG.set(cli.debug) {
        bail!("Failed to set DEBUG variable");
    }

    let (cols, rows) = size()?;
    terminal_setup(rows, cols)?;

    panic_hook::add_panic_hook(&cleanup);

    let data = setup_logging_and_data()?;

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
        Cmd::new(">", indent),
        Cmd::new("<", unindent),
    ];

    let operators: &[Operator] = &[
        Operator::new('d', delete),
        Operator::new('y', yank),
        Operator::new('t', change),
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

    // Used for not putting excluded chars in the chain
    let all_normal_chars =
        enumerate_normal_chars(commands, operators, motions, text_objects, view_commands);

    let mut view: View = View::new(cols, rows);
    let global_state = GlobalState::new();

    if !data.has_opened && path.is_none() {
        view.get_view_box().write_welcome_screen();
        write_data(&"has_opened", &"true");
    }

    view.set_path(path);
    view.load_file()?;

    let _ = view.get_view_box().parse();

    view.flush(&global_state, false)?;

    program_loop(
        commands,
        operators,
        motions,
        text_objects,
        view_commands,
        &all_normal_chars,
        global_state,
        view,
    )?;

    cleanup()
}
