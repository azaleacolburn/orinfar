use crate::{
    mode::Mode, operator::Operator, register::RegisterHandler, status_bar::StatusBar,
    text_object::TextObjectType, undo::UndoTree,
};

pub struct GlobalState<'a> {
    pub next_operation: Option<&'a Operator>,
    pub text_object_type: Option<TextObjectType>,

    pub mode: Mode,
    pub count: u32,
    pub chained: Vec<char>,
    pub search_str: Vec<char>,

    pub undo_tree: UndoTree,
    pub register_handler: RegisterHandler,
    pub status_bar: StatusBar,
}

impl GlobalState<'_> {
    pub fn new() -> Self {
        Self {
            next_operation: None,
            text_object_type: None,

            mode: Mode::Normal,
            count: 1,
            chained: Vec::new(),
            search_str: Vec::new(),

            undo_tree: UndoTree::new(),
            register_handler: RegisterHandler::new(),
            status_bar: StatusBar::new(),
        }
    }
}
