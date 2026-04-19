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

impl View {
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

    pub fn move_view_box(&mut self, f: fn(&mut Self) -> Option<usize>) {
        if let Some(i) = f(self) {
            self.cursor = i;
        }
    }

    pub fn move_down_one_view_box(&mut self) {
        self.move_view_box(Self::vb_down);
    }

    pub fn move_up_one_view_box(&mut self) {
        self.move_view_box(Self::vb_up);
    }

    pub fn move_left_one_view_box(&mut self) {
        self.move_view_box(Self::vb_left);
    }

    pub fn move_right_one_view_box(&mut self) {
        self.move_view_box(Self::vb_right);
    }

    pub fn split_curr_view_box_vertical(&mut self) {
        self.split_view_box_vertical(self.cursor);
    }

    pub fn split_curr_view_box_horizontal(&mut self) {
        self.split_view_box_horizontal(self.cursor);
    }
}
