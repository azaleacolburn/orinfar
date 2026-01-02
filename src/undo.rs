use crate::buffer::Buffer;
use crate::{DEBUG, io::log};

#[derive(Debug, Clone)]
pub enum Action {
    Insert {
        position: usize,
        text: String,
    },
    Delete {
        position: usize,
        text: String,
    },
    Replace {
        positions: Vec<usize>,
        original: String,
        new: String,
    },
}
impl Action {
    pub fn delete(final_position: usize, text: impl ToString) -> Self {
        Action::Delete {
            position: final_position,
            text: text.to_string(),
        }
    }

    pub fn insert(initial_position: usize, text: impl ToString) -> Self {
        Action::Insert {
            position: initial_position,
            text: text.to_string(),
        }
    }

    // Original and new must be of the same length
    pub fn replace(
        initial_positions: Vec<usize>,
        original: impl ToString,
        new: impl ToString,
    ) -> Self {
        Action::Replace {
            positions: initial_positions,
            original: original.to_string(),
            new: new.to_string(),
        }
    }
}

/// Handles the tracking of versions of the buffer
/// This could be through managing actions and letting every actions have an inverse (I think this
/// is the way to go)
/// To start with, the tree will not support redoing
#[derive(Debug, Clone)]
pub struct UndoTree {
    actions: Vec<Action>,
}

impl UndoTree {
    pub fn new() -> Self {
        UndoTree {
            actions: Vec::new(),
        }
    }

    pub fn undo(&mut self, buffer: &mut Buffer) {
        let action = match self.actions.pop() {
            Some(prev_state) => prev_state,
            None => return,
        };

        match action {
            Action::Insert { text, position } => {
                buffer.cursor = position;
                (0..text.len()).for_each(|_| buffer.delete_curr_char());
            }
            Action::Delete { text, position } => {
                buffer.cursor = position;
                text.chars().rev().for_each(|c| buffer.insert_char(c));
            }
            Action::Replace {
                positions,
                original,
                new,
            } => {
                log!("un-replacing: {} {}", buffer.cursor, new);
                assert_eq!(original.len(), new.len());
                buffer.replace_text(original, new, &positions, self);
            }
        }

        buffer.update_list_set(.., true);
        buffer.has_changed = true;
    }

    pub fn new_action_merge(&mut self, mut action: Action) {
        // The point of this is to squash keystrokes
        match &action {
            Action::Insert { text, position: _ } => {
                let mut text = text.clone();

                if let Some(last) = self.actions.clone().last()
                    && let Action::Insert {
                        text: last_text,
                        position: last_position,
                    } = &last
                {
                    text.push_str(last_text);
                    self.actions.pop();
                    action = Action::insert(*last_position, text);
                }
            }
            Action::Delete { text, position } => {
                let mut text = text.clone();
                if let Some(last) = self.actions.clone().last()
                    && let Action::Delete {
                        text: last_text,
                        position: _,
                    } = &last
                {
                    text.push_str(last_text);
                    self.actions.pop();
                    action = Action::delete(*position, text);
                }
            }
            Action::Replace {
                positions,
                original,
                new,
            } => {
                self.actions.pop();
                let positions = positions.iter().chain();
                action = Action::replace(*position, text);
            }
            _ => {}
        };

        self.actions.push(action);
    }

    pub fn new_action(&mut self, action: Action) {
        self.actions.push(action);
    }
}
