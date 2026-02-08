use crate::{
    DEBUG,
    commands::Command,
    log,
    mode::Mode,
    motion::Motion,
    operator::Operator,
    register::RegisterHandler,
    text_object::{self, TextObject, TextObjectType},
    undo::UndoTree,
    utility::last_char,
    view::View,
    view_command::ViewCommand,
};

#[allow(clippy::too_many_arguments)]
pub fn match_action<'a>(
    chained: &mut Vec<char>,
    next_operation: &mut Option<&'a Operator<'a>>,
    text_object_type: &mut Option<TextObjectType>,
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
    text_objects: &[TextObject],
    view_commands: &[ViewCommand],
) {
    let last = *match chained.last() {
        Some(c) => c,
        None => return,
    };
    let buffer = view.get_buffer_mut();

    if let Some(operation) = next_operation {
        if let Some(motion) = motions.iter().find(|motion| last_char(motion.name) == last) {
            (0..*count).for_each(|_| {
                operation.execute_motion(motion, buffer, register_handler, mode, undo_tree);
            });

            reset(chained, count, next_operation, last_chained, last_count);
        } else if last == 'i' {
            *text_object_type = Some(TextObjectType::Inside);
        } else if last == 'a' {
            *text_object_type = Some(TextObjectType::Around);
        } else if last_char(operation.name) == last {
            (0..*count).for_each(|_| {
                operation.entire_line(buffer, register_handler, mode, undo_tree);
            });

            reset(chained, count, next_operation, last_chained, last_count);
        } else if let Some(to_type) = text_object_type {
            if let Some(text_object) = text_objects.iter().find(|to| last_char(to.name) == last) {
                (0..*count).for_each(|_| {
                    operation.execute_text_object(
                        text_object,
                        to_type,
                        buffer,
                        register_handler,
                        mode,
                        undo_tree,
                    );
                });

                reset(chained, count, next_operation, last_chained, last_count);
            }
        }
    } else if let Some(command) = commands
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
    } else if chained.len() == 1
        && let Some(motion) = motions
            .iter()
            .find(|motion| motion.name.chars().last().unwrap() == last)
    {
        (0..*count).for_each(|_| {
            motion.apply(buffer);
        });

        reset(chained, count, next_operation, last_chained, last_count);
    } else if let Some(operator) = operators
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
    last_chained.clone_from(chained);
    *last_count = *count;

    chained.clear();
    *count = 1;
    *next_operation = None;
}

pub fn enumerate_normal_chars(
    commands: &[Command],
    operators: &[Operator],
    motions: &[Motion],
    text_objects: &[TextObject],
    view_commands: &[ViewCommand],
) -> Vec<char> {
    let command_chars = commands.iter().flat_map(|cmd| cmd.name.chars());
    let operator_chars = operators.iter().flat_map(|cmd| cmd.name.chars());
    let motion_chars = motions.iter().flat_map(|cmd| cmd.name.chars());
    let text_object_chars = text_objects.iter().flat_map(|cmd| cmd.name.chars());
    let view_command_chars = view_commands.iter().flat_map(|cmd| cmd.name.chars());

    command_chars
        .chain(operator_chars)
        .chain(motion_chars)
        .chain(text_object_chars)
        .chain(view_command_chars)
        .collect()
}
