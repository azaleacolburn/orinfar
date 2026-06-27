pub struct QuickFix {
    start_position: usize,
    end_position: usize,
    buffer_idx: usize,
}

// TODO Figure out how to display this. Maybe just hold the viewbox index for displaying the list
// But really, there should always be a view box for the list???
pub struct QuickFixList {
    fixes: Vec<QuickFix>,
    display_buffer_idx: Option<usize>,
}

impl QuickFixList {
    pub fn new() -> Self {
        QuickFixList {
            fixes: vec![],
            display_buffer_idx: None,
        }
    }
}
