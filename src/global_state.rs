use crate::{mode::Mode, operator::Operator, text_object::TextObjectType};

pub struct GlobalState<'a> {
    pub next_operation: Option<&'a Operator<'a>>,
    pub text_object_type: Option<TextObjectType>,

    pub mode: Mode,
    pub count: u32,
    pub chained: Vec<char>,
    pub search_str: Vec<char>,
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
        }
    }
}
