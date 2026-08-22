use crate::{
    COMMANDS, MOTIONS, OPERATORS, TEXT_OBJECTS, VIEW_COMMANDS, buffer::Buffer,
    global_state::GlobalState, operator::Operator, text_object::TextObjectType, utility::last_char,
    view::View,
};

pub fn match_action(
    global_state: &mut GlobalState,
    last_chained: &mut Vec<char>,
    last_count: &mut u32,
    view: &mut View,
) {
    let Some(last) = global_state.chained.last() else {
        return;
    };

    let cmd: String = global_state.chained.iter().collect();

    let buffer = view.get_buffer_mut();

    if let Some(operation) = global_state.next_operation {
        handle_pending_operation(
            operation,
            buffer,
            global_state,
            last_chained,
            last_count,
            *last,
        );
    } else if let Some(command) = COMMANDS.iter().find(|motion| motion.name == cmd) {
        (0..global_state.count).for_each(|_| {
            command.execute(
                buffer,
                &mut global_state.register_handler,
                &mut global_state.mode,
                &mut global_state.undo_tree,
            );
        });

        reset(global_state, last_chained, last_count);
    } else if let Some(view_command) = VIEW_COMMANDS.iter().find(|command| command.name == cmd) {
        (0..global_state.count).for_each(|_| view_command.execute(view));

        reset(global_state, last_chained, last_count);
    } else if global_state.chained.len() == 1
        && let Some(motion) = MOTIONS.iter().find(|motion| motion.name == *last)
    {
        (0..global_state.count).for_each(|_| motion.apply(buffer));

        reset(global_state, last_chained, last_count);
    } else if let Some(operator) = OPERATORS.iter().find(|operator| operator.name == *last) {
        global_state.next_operation = Some(operator);
    }
}

fn handle_pending_operation(
    operation: &Operator,
    buffer: &mut Buffer,
    global_state: &mut GlobalState,
    last_chained: &mut Vec<char>,
    last_count: &mut u32,
    last: char,
) {
    if last == 'i' {
        global_state.text_object_type = Some(TextObjectType::Inside);
    } else if last == 'a' {
        global_state.text_object_type = Some(TextObjectType::Around);
    } else if operation.name == last {
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
        let Some(text_object) = TEXT_OBJECTS.iter().find(|to| last_char(to.name) == last) else {
            // TODO Decide whether we should log things triggered easily by users?
            // log!("Could not find text object {}", last);
            return;
        };

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
    } else if let Some(motion) = MOTIONS.iter().find(|motion| motion.name == last) {
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
}

pub fn reset(global_state: &mut GlobalState, last_chained: &mut Vec<char>, last_count: &mut u32) {
    last_chained.clone_from(&global_state.chained);
    *last_count = global_state.count;

    global_state.chained.clear();
    global_state.count = 1;
    global_state.next_operation = None;
}

pub fn enumerate_normal_chars() -> Vec<char> {
    let command_chars = COMMANDS.iter().flat_map(|cmd| cmd.name.chars());
    let operator_chars = OPERATORS.iter().map(|cmd| cmd.name);
    let motion_chars = MOTIONS.iter().map(|cmd| cmd.name);
    let text_object_chars = TEXT_OBJECTS.iter().flat_map(|cmd| cmd.name.chars());
    let view_command_chars = VIEW_COMMANDS.iter().flat_map(|cmd| cmd.name.chars());

    command_chars
        .chain(operator_chars)
        .chain(motion_chars)
        .chain(text_object_chars)
        .chain(view_command_chars)
        .collect()
}
