use crate::{DEBUG, buffer::Buffer, log, view::View, view_box::ViewBox};

pub struct ViewCommand<'a> {
    pub name: &'a str,
    command: fn(view: &mut View),
}

impl<'a> ViewCommand<'a> {
    pub fn new(name: &'a str, command: fn(view: &mut View)) -> Self {
        ViewCommand { name, command }
    }

    pub fn execute(&self, view: &mut View) {
        (self.command)(view);
    }
}

pub fn center_viewbox_on_cursor(view: &mut View) {
    let view_box = view.get_view_box();

    let half_height = view_box.height() as usize / 2;
    let row = view_box.buffer.get_row();
    if row < half_height {
        return;
    }

    let new_top = row - half_height;
    view_box.top = new_top;

    view_box.buffer.update_list_set(.., true);
    view_box.buffer.has_changed = true;
}

pub fn move_down_one_view_box(view: &mut View) {
    let view_box = view.get_view_box();
    let (x, y) = view_box.get_lower_left();
    let predicate = |view_box: &ViewBox| -> bool { view_box.x == x && view_box.y == y };

    if let Some(i) = view.position_of_box(predicate) {
        log!("Found lower view box");
        view.cursor = i;
    }
}
