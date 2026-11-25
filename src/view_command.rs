use crate::{view::View, view_box::ViewBox};

pub struct ViewCommand<'a> {
    pub name: &'a str,
    command: fn(view: &mut View),
}

impl<'a> ViewCommand<'a> {
    pub const fn new(name: &'a str, command: fn(view: &mut View)) -> Self {
        ViewCommand { name, command }
    }

    pub fn execute(&self, view: &mut View) {
        (self.command)(view);
    }
}

pub fn center_viewbox_on_cursor(view: &mut View) {
    let view_box: &mut ViewBox = view.get_view_box_mut();

    let Some(ref render_info) = view_box.cached_render_info else {
        return;
    };

    let half_height = render_info.height as usize / 2;
    let row = view_box.buffer.get_row();
    if row < half_height {
        return;
    }

    let new_top = row - half_height;
    view_box.top = new_top;

    view_box.buffer.update_list_set(.., true);
    view_box.buffer.has_changed = true;
}
