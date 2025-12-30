use crate::{buffer::Buffer, view_box::ViewBox};

pub struct ViewCommand<'a> {
    pub name: &'a str,
    command: fn(buffer: &mut Buffer, view_box: &mut ViewBox),
}

impl<'a> ViewCommand<'a> {
    pub fn new(name: &'a str, command: fn(buffer: &mut Buffer, view_box: &mut ViewBox)) -> Self {
        ViewCommand { name, command }
    }

    pub fn execute(&self, buffer: &mut Buffer, view_box: &mut ViewBox) {
        (self.command)(buffer, view_box)
    }
}

pub fn center_viewbox_on_cursor(buffer: &mut Buffer, view_box: &mut ViewBox) {
    let half_height = view_box.height() / 2;
    let row = buffer.get_row();
    if row < half_height {
        return;
    }

    let new_top = row - half_height;
    view_box.top = new_top;

    buffer.update_list_set(.., true);
    buffer.has_changed = true;
}
