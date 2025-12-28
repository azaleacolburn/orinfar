use crate::{buffer::Buffer, utility::log};

#[derive(Debug, Clone)]
pub enum ActionType {
    Insert(String),
    Delete(String),
    Replace { original: String, new: String },
    Neither,
}

#[derive(Debug, Clone)]
pub struct Action {
    // It's an initial position for the Insertions
    // And a final position for the Deletions
    position: usize,
    r#type: ActionType,
}

impl Action {
    pub fn delete(initial_position: usize, text: impl ToString) -> Self {
        Action {
            position: initial_position,
            r#type: ActionType::Delete(text.to_string()),
        }
    }

    pub fn insert(initial_position: usize, text: impl ToString) -> Self {
        Action {
            position: initial_position,
            r#type: ActionType::Insert(text.to_string()),
        }
    }

    // Original and new must be of the same length
    pub fn replace(initial_position: usize, original: impl ToString, new: impl ToString) -> Self {
        Action {
            position: initial_position,
            r#type: ActionType::Replace {
                original: original.to_string(),
                new: new.to_string(),
            },
        }
    }

    pub fn neither(initial_position: usize) -> Self {
        Action {
            position: initial_position,
            r#type: ActionType::Neither,
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
        log("here1");

        match action.r#type {
            ActionType::Insert(text) => {
                buffer.cursor = action.position;
                log(format!("here5: {}", text.len()));
                (0..text.len()).for_each(|_| buffer.delete_curr_char());
                log("here7");
            }
            ActionType::Delete(text) => {
                buffer.cursor = action.position;
                text.chars().rev().for_each(|c| buffer.insert_char(c));
            }
            ActionType::Replace { original, new } => {
                buffer.cursor = action.position;
                assert_eq!(original.len(), new.len());
                new.chars().for_each(|c| buffer.replace_curr_char(c));
            }
            ActionType::Neither => buffer.cursor = action.position,
        }

        buffer.update_list_set(.., true);
        buffer.has_changed = true;
    }

    pub fn new_action(&mut self, mut action: Action) {
        // The point of this is to squash keystrokes
        log("undo");
        log(format!("action: {:?}", action));

        match &action.r#type {
            ActionType::Insert(text) => {
                let mut text = text.clone();

                if let Some(last) = self.actions.clone().last() {
                    if let ActionType::Insert(last_text) = &last.r#type {
                        text.push_str(&last_text);
                        self.actions.pop();
                        action = Action::insert(last.position, text);
                    }
                }
            }
            ActionType::Delete(text) => {
                let mut text = text.clone();
                if let Some(last) = self.actions.clone().last() {
                    if let ActionType::Delete(last_text) = &last.r#type {
                        text.push_str(&last_text);
                        self.actions.pop();
                        action = Action::delete(action.position, text);
                    }
                }
            }
            _ => {}
        };

        self.actions.push(action);
    }
}
