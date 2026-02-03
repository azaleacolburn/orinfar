use crate::{
    DEBUG, commands::Command, log, mode::Mode, motion::Motion, operator::Operator,
    register::RegisterHandler, undo::UndoTree, utility::last_char, view::View,
    view_command::ViewCommand,
};

#[allow(clippy::too_many_arguments)]
pub fn match_action<'a>(
    chained: &mut Vec<char>,
    next_operation: &mut Option<&'a Operator<'a>>,
    count: &mut u16,

    last_chained: &mut Vec<char>,
    last_count: &mut u16,

    register_handler: &mut RegisterHandler,
    undo_tree: &mut UndoTree,
    view: &mut View,
    mode: &mut Mode,

    commands: &[Command],
    operators: &'a [Operator<'a>],
    motions: &[Motion],
    view_commands: &[ViewCommand],
) {
    log!("count in match action: {}", count);

    let last = match chained.last() {
        Some(c) => c,
        None => return,
    }
    .clone();
    let buffer = view.get_buffer_mut();

    if let Some(command) = commands
        .iter()
        .find(|motion| last_char(motion.name) == last)
    {
        (0..*count).for_each(|_| {
            command.execute(buffer, register_handler, mode, undo_tree);
        });

        reset(chained, count, next_operation, last_chained, last_count);
    } else if let Some(view_command) = view_commands
        .iter()
        .find(|command| command.name == chained.iter().collect::<String>())
    {
        (0..*count).for_each(|_| {
            view_command.execute(view);
        });

        reset(chained, count, next_operation, last_chained, last_count);
    } else if let Some(operation) = next_operation {
        if let Some(motion) = motions.iter().find(|motion| last_char(motion.name) == last) {
            (0..*count).for_each(|_| {
                operation.execute(motion, buffer, register_handler, mode, undo_tree);
            });

            reset(chained, count, next_operation, last_chained, last_count);
        } else if last_char(operation.name) == last {
            (0..*count).for_each(|_| {
                operation.entire_line(buffer, register_handler, mode, undo_tree);
            });

            reset(chained, count, next_operation, last_chained, last_count);
        }
    } else if chained.len() == 1
        && let Some(motion) = motions
            .iter()
            .find(|motion| motion.name.chars().last().unwrap() == last)
    {
        (0..*count).for_each(|_| {
            motion.apply(buffer);
        });

        reset(chained, count, next_operation, last_chained, last_count);
    }
    if let Some(operator) = operators
        .iter()
        .find(|operator| last_char(operator.name) == last)
    {
        *next_operation = Some(operator);
    }
}

pub fn reset<'a>(
    chained: &mut Vec<char>,
    count: &mut u16,
    next_operation: &mut Option<&'a Operator<'a>>,
    last_chained: &mut Vec<char>,
    last_count: &mut u16,
) {
    log!("resetting count: {}", count);
    *last_chained = chained.to_vec();
    *last_count = *count;

    chained.clear();
    *count = 1;
    *next_operation = None;
}

pub fn enumerate_normal_chars(
    commands: &[Command],
    operators: &[Operator],
    motions: &[Motion],
    view_commands: &[ViewCommand],
) -> Vec<char> {
    let command_chars = commands.iter().flat_map(|cmd| cmd.name.chars());
    let operator_chars = operators.iter().flat_map(|cmd| cmd.name.chars());
    let motion_chars = motions.iter().flat_map(|cmd| cmd.name.chars());
    let view_command_chars = view_commands.iter().flat_map(|cmd| cmd.name.chars());

    command_chars
        .chain(operator_chars)
        .chain(motion_chars)
        .chain(view_command_chars)
        .collect()
}
