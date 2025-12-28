use crate::buffer::Buffer;

pub enum ActionType {
    Insert(String),
    Delete(String),
    Replace { original: String, new: String },
    Neither,
}

pub struct Action {
    initial_position: usize,
    r#type: ActionType,
}

impl Action {
    pub fn delete(initial_position: usize, text: impl ToString) -> Self {
        Action {
            initial_position,
            r#type: ActionType::Delete(text.to_string()),
        }
    }

    pub fn insert(initial_position: usize, text: impl ToString) -> Self {
        Action {
            initial_position,
            r#type: ActionType::Insert(text.to_string()),
        }
    }

    // Original and new must be of the same length
    pub fn replace(initial_position: usize, original: impl ToString, new: impl ToString) -> Self {
        Action {
            initial_position,
            r#type: ActionType::Replace {
                original: original.to_string(),
                new: new.to_string(),
            },
        }
    }

    pub fn neither(initial_position: usize) -> Self {
        Action {
            initial_position,
            r#type: ActionType::Neither,
        }
    }
}

/// Handles the tracking of versions of the buffer
/// This could be through managing actions and letting every actions have an inverse (I think this
/// is the way to go)
/// To start with, the tree will not support redoing
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

        match action.r#type {
            ActionType::Insert(text) => {
                buffer.cursor = action.initial_position;
                (0..text.len()).for_each(|_| buffer.delete_curr_char());
            }
            ActionType::Delete(text) => {
                buffer.cursor = action.initial_position;
                text.chars().for_each(|c| buffer.insert_char(c));
            }
            ActionType::Replace { original, new } => {
                buffer.cursor = action.initial_position;
                assert_eq!(original.len(), new.len());
                new.chars().for_each(|c| buffer.replace_curr_char(c));
            }
            ActionType::Neither => buffer.cursor = action.initial_position,
        }

        buffer.update_list_set(.., true);
        buffer.has_changed = true;
    }

    pub fn new_action(&mut self, mut action: Action) {
        // The point of this is to squash keystrokes

        match &action.r#type {
            ActionType::Insert(text) => {
                let mut text = text.clone();
                let mut first_pos = action.initial_position;
                for action in self.actions.iter().rev() {
                    if let ActionType::Insert(t) = &action.r#type {
                        first_pos = action.initial_position;
                        text.push_str(&t);
                    } else {
                        break;
                    }
                }

                action = Action::insert(first_pos, text);
            }
            ActionType::Delete(text) => {
                let mut text = text.clone();
                let mut first_pos = action.initial_position;
                for action in self.actions.iter().rev() {
                    if let ActionType::Insert(t) = &action.r#type {
                        first_pos = action.initial_position;
                        text.push_str(&t);
                    } else {
                        break;
                    }
                }

                action = Action::delete(first_pos, text);
            }
            _ => {}
        };

        self.actions.push(action);
    }
}
