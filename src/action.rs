use crate::{
    commands::Command,
    global_state::GlobalState,
    motion::Motion,
    operator::Operator,
    text_object::{TextObject, TextObjectType},
    utility::last_char,
    view::View,
    view_command::ViewCommand,
};

#[allow(clippy::too_many_arguments)]
pub fn match_action<'a>(
    global_state: &mut GlobalState<'a>,

    last_chained: &mut Vec<char>,
    last_count: &mut u32,

    view: &mut View,

    commands: &[Command],
    operators: &'a [Operator<'a>],
    motions: &[Motion],
    text_objects: &[TextObject],
    view_commands: &[ViewCommand],
) {
    let last = *match global_state.chained.last() {
        Some(c) => c,
        None => return,
    };
    let buffer = view.get_buffer_mut();

    let cmd: String = global_state.chained.iter().collect();

    if let Some(operation) = global_state.next_operation {
        if last == 'i' {
            global_state.text_object_type = Some(TextObjectType::Inside);
        } else if last == 'a' {
            global_state.text_object_type = Some(TextObjectType::Around);
        } else if last_char(operation.name) == last {
            (0..global_state.count).for_each(|_| {
                operation.entire_line(
                    buffer,
                    &mut global_state.register_handler,
                    &mut global_state.mode,
                    &mut global_state.undo_tree,
                );
            });

            reset(global_state, last_chained, last_count);
        } else if let Some(to_type) = &global_state.text_object_type {
            // NOTE
            // This is fine because for the text object, we only care about the last key pressed
            if let Some(text_object) = text_objects.iter().find(|to| last_char(to.name) == last) {
                (0..global_state.count).for_each(|_| {
                    operation.execute_text_object(
                        text_object,
                        to_type,
                        buffer,
                        &mut global_state.register_handler,
                        &mut global_state.mode,
                        &mut global_state.undo_tree,
                    );
                });

                global_state.text_object_type = None;

                reset(global_state, last_chained, last_count);
            }
        } else if let Some(motion) = motions.iter().find(|motion| motion.name == last) {
            (0..global_state.count).for_each(|_| {
                operation.execute_motion(
                    motion,
                    buffer,
                    &mut global_state.register_handler,
                    &mut global_state.mode,
                    &mut global_state.undo_tree,
                );
            });

            reset(global_state, last_chained, last_count);
        }
    } else if let Some(command) = commands.iter().find(|motion| motion.name == cmd) {
        (0..global_state.count).for_each(|_| {
            command.execute(
                buffer,
                &mut global_state.register_handler,
                &mut global_state.mode,
                &mut global_state.undo_tree,
            );
        });

        reset(global_state, last_chained, last_count);
    } else if let Some(view_command) = view_commands.iter().find(|command| command.name == cmd) {
        (0..global_state.count).for_each(|_| {
            view_command.execute(view);
        });

        reset(global_state, last_chained, last_count);
    } else if global_state.chained.len() == 1
        && let Some(motion) = motions.iter().find(|motion| motion.name == last)
    {
        (0..global_state.count).for_each(|_| {
            motion.apply(buffer);
        });

        reset(global_state, last_chained, last_count);
    } else if let Some(operator) = operators
        .iter()
        .find(|operator| last_char(operator.name) == last)
    {
        global_state.next_operation = Some(operator);
    }
}

pub fn reset(global_state: &mut GlobalState, last_chained: &mut Vec<char>, last_count: &mut u32) {
    last_chained.clone_from(&global_state.chained);
    *last_count = global_state.count;

    global_state.chained.clear();
    global_state.count = 1;
    global_state.next_operation = None;
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
    let motion_chars = motions.iter().map(|cmd| cmd.name);
    let text_object_chars = text_objects.iter().flat_map(|cmd| cmd.name.chars());
    let view_command_chars = view_commands.iter().flat_map(|cmd| cmd.name.chars());

    command_chars
        .chain(operator_chars)
        .chain(motion_chars)
        .chain(text_object_chars)
        .chain(view_command_chars)
        .collect()
}
