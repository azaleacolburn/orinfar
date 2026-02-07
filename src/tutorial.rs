use crate::{
    utility::{count_line, count_longest_line},
    view_box::ViewBox,
};
use ropey::Rope;

const WELCOME_TEXT: &str = r#"Welcome To Orinfar
The Text Editor For Witches
This screen will only appear the first time you read it, so stick around until the end.

This is a modal editor with similar behavior to VI, but there are several key differences.
You should read our [USER MANUAL](https://github.com/azaleacolburn/orinfar/blob/main/docs/MANUAL.md) before trying to edit any real projects.

If you have any bugs to report, features to suggest, documentation updates, or just want to get involved,
    please check out our [GITHUB REPOSITORY](https://github.com/azaleacolburn/orinfar)
"#;

const WELCOME_HEIGHT: u16 = count_line(WELCOME_TEXT);
const WELCOME_WIDTH: u16 = count_longest_line(WELCOME_TEXT);

impl ViewBox {
    pub fn write_welcome_screen(&mut self) {
        let vertical_padding: i32 = (self.height - WELCOME_HEIGHT).into();
        let max_horizontal_padding: i32 = (self.width - WELCOME_WIDTH).into();

        if vertical_padding <= 0 || max_horizontal_padding <= 0 {
            return;
        }

        let mut contents = String::new();
        for _ in 0..vertical_padding / 2 {
            contents.push('\n');
        }
        for line in WELCOME_TEXT.lines() {
            write_line_centered(line, &mut contents, self.width);
        }

        self.buffer.rope = Rope::from(contents);
        (0..vertical_padding as u16 / 2 + WELCOME_HEIGHT)
            .for_each(|_| self.buffer.update_list_add_current());
        self.buffer.has_changed = true;
    }
}

fn write_line_centered(line: &str, contents: &mut String, width: u16) {
    let padding = (0..(width - line.len() as u16) / 2)
        .map(|_| ' ')
        .collect::<String>();
    contents.push_str(&padding);
    contents.push_str(line.trim_matches('\n'));
    contents.push_str(&padding);
    contents.push('\n');
}
