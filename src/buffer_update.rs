use std::{ops::RangeBounds, slice::SliceIndex};

use crate::{buffer::Buffer, log};

impl Buffer {
    /// Updates the has_changed list when a `\n` character is removed
    pub fn update_list_remove_current(&mut self) {
        let current_line = self.get_row();
        self.update_list_remove(current_line);
        self.update_list_set(usize::max(current_line, 1) - 1.., true);
        self.has_changed = true;
        log(format!("update_list: {:?}", self.lines_for_updating))
    }

    /// Updates the has_changed list when a `\n` character is added
    pub fn update_list_add_current(&mut self) {
        let current_line = self.get_row();
        self.lines_for_updating.insert(current_line, true);
        self.update_list_set(current_line.., true);
        self.has_changed = true;
    }

    /// Updates the has_changed list when a `\n` character is removed at the given line number
    pub fn update_list_remove(&mut self, idx: usize) {
        self.lines_for_updating.remove(idx);
        self.update_list_set(idx.., true);
        self.has_changed = true;
    }

    pub fn update_list_add(&mut self, idx: usize) {
        self.lines_for_updating.insert(idx, true);
        self.update_list_set(idx.., true);
        self.has_changed = true;
    }

    pub fn update_list_use_current_line(&mut self) {
        let current_line = self.get_row();
        self.lines_for_updating[current_line] = true;
        self.has_changed = true;
    }

    pub fn update_list_reset(&mut self) {
        self.update_list_set(.., false);
        self.has_changed = false;
    }

    /// Does not set `has_changed`
    pub fn update_list_set<R: RangeBounds<usize> + SliceIndex<[bool], Output = [bool]>>(
        &mut self,
        range: R,
        value: bool,
    ) {
        self.lines_for_updating[range]
            .iter_mut()
            .for_each(|b| *b = value);
    }
}
