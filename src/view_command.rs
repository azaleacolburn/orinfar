use crate::view::View;

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

    let half_height = view_box.height as usize / 2;
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
    if let Some(i) = view.position_view_box_down() {
        view.cursor = i;
    }
}
pub fn move_up_one_view_box(view: &mut View) {
    if let Some(i) = view.position_view_box_up() {
        view.cursor = i;
    }
}

pub fn move_left_one_view_box(view: &mut View) {
    if let Some(i) = view.position_view_box_left() {
        view.cursor = i;
    }
}
pub fn move_right_one_view_box(view: &mut View) {
    if let Some(i) = view.position_view_box_right() {
        view.cursor = i;
    }
}

pub fn delete_curr_view_box(view: &mut View) {
    view.delete_curr_view_box();
}

pub fn split_curr_view_box_vertical(view: &mut View) {
    view.split_view_box_vertical(view.cursor);
}

pub fn split_curr_view_box_horizontal(view: &mut View) {
    view.split_view_box_horizontal(view.cursor);
}
