use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

#[derive(Debug, Clone)]
pub struct QuickFix {
    pub start_position: usize,
    pub end_position: usize,
    pub file_path: Option<PathBuf>,
    pub buffer_idx: Option<usize>,
}

impl QuickFix {
    pub fn new(
        start_position: usize,
        end_position: usize,
        file_path: Option<PathBuf>,
        buffer_idx: Option<usize>,
    ) -> Self {
        QuickFix {
            start_position,
            end_position,
            file_path,
            buffer_idx,
        }
    }
}

// TODO Figure out how to display this. Maybe just hold the viewbox index for displaying the list
// But really, there should always be a view box for the list???
#[derive(Debug, Clone)]
pub struct QuickFixList {
    pub fixes: Vec<QuickFix>,
    display_buffer_idx: usize,
}

impl QuickFixList {
    pub fn new(display_buffer_idx: usize) -> Self {
        QuickFixList {
            fixes: vec![],
            display_buffer_idx,
        }
    }

    pub fn push_opened(&mut self, start: usize, end: usize, file_path: PathBuf, buffer_idx: usize) {
        let fix = QuickFix::new(start, end, Some(file_path), Some(buffer_idx));

        self.fixes.push(fix);
    }

    pub fn push_unopened(&mut self, start: usize, end: usize, file_path: PathBuf) {
        let fix = QuickFix::new(start, end, Some(file_path), None);

        self.fixes.push(fix);
    }

    pub fn push_unattached(&mut self, start: usize, end: usize, buffer_idx: usize) {
        let fix = QuickFix::new(start, end, None, Some(buffer_idx));

        self.fixes.push(fix);
    }
}

impl Deref for QuickFixList {
    type Target = Vec<QuickFix>;

    fn deref(&self) -> &Self::Target {
        &self.fixes
    }
}

impl DerefMut for QuickFixList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.fixes
    }
}
