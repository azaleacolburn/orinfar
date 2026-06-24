use crate::{
    undo::UndoTree,
    utility::{count_lines, count_longest_line},
    view_box::ViewBox,
};

const WELCOME_TEXT: &str = r"Welcome To Orinfar

The Text Editor For Witches

This screen will only appear the first time you read it, so stick around until the end.

This is a modal editor with similar behavior to VI, but there are several key differences.

You should read our [USER MANUAL](https://github.com/azaleacolburn/orinfar/blob/main/docs/MANUAL.md)
before trying to edit any real projects.

If you have any bugs to report, features to suggest, documentation updates, or just want to get involved,
    please check out our [GITHUB REPOSITORY](https://github.com/azaleacolburn/orinfar)
";

const WELCOME_HEIGHT: u16 = count_lines(WELCOME_TEXT);
const WELCOME_WIDTH: u16 = count_longest_line(WELCOME_TEXT);

impl ViewBox {
    pub fn write_welcome_screen(&mut self, undo_tree: &mut UndoTree) {
        let vertical_padding = i32::from(self.height) - i32::from(WELCOME_HEIGHT);
        let max_horizontal_padding = i32::from(self.width) - i32::from(WELCOME_WIDTH);

        if vertical_padding < 0 || max_horizontal_padding < 0 {
            return;
        }

        let mut output = String::new();
        for _ in 0..vertical_padding / 2 - 1 {
            output.push('\n');
        }
        for line in WELCOME_TEXT.lines() {
            output.push('\n');
            self.write_line_centered(&mut output, line, self.width);
        }

        self.buffer.replace_contents(&output, undo_tree);
    }

    fn write_line_centered(&self, output: &mut String, line: &str, width: u16) {
        let leftover = usize::from(width) - line.len() - self.left_padding();
        let padding = (0..leftover / 2).map(|_| ' ').collect::<String>();

        output.push_str(&padding);
        output.push_str(line.trim_matches('\n'));
        output.push_str(&padding);
    }
}
