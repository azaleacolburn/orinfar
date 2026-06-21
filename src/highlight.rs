use crate::{
    utility::{print_tree, traverse_tree},
    view_box::ViewBox,
};
use crossterm::style::Color;
use tree_sitter::{Node, Point, Tree, TreeCursor};

impl ViewBox {
    pub fn parse(&mut self) -> Option<&Tree> {
        if self.buffer.has_changed {
            let source: Vec<u8> = self.buffer.rope.bytes().collect();
            // TODO
            // Edit the `Tree` and pass it to the `parse` function
            let (parser, _language) = self.parser.as_mut()?;
            self.parse_tree = parser.parse(source, None);
        }

        self.parse_tree.as_ref()
    }

    /// Adds hl blocks for empty lines at the end of the document
    pub fn append_empty_lines(&self, tree_hl_blocks: &mut Vec<Vec<HLBlock>>) {
        let buffer_lines: usize = self.buffer.rope.len_lines();
        let hl_lines: usize = tree_hl_blocks.len();
        if hl_lines < buffer_lines {
            for _ in hl_lines..buffer_lines {
                tree_hl_blocks.push(vec![HLBlock::empty()]);
            }
        }
    }

    /// Adds blank hl blocks for empty lines in the middle of the document
    pub fn fill_in_empty_lines(tree_hl_blocks: &mut [Vec<HLBlock>]) {
        tree_hl_blocks
            .iter_mut()
            .filter(|l| l.is_empty())
            .for_each(|hl_blocks| {
                hl_blocks.push(HLBlock::empty());
            });
    }

    /// Returns a list of lines, each containing highlight blocks
    /// Returns an empty list if `self.parse_tree.is_none()`
    pub fn highlight(&self) -> Vec<Vec<HLBlock>> {
        let Some(tree) = &self.parse_tree else {
            return vec![];
        };

        let Some((_parser, language)) = &self.parser else {
            return vec![];
        };

        let mut tree_hl_blocks = highlight_tree(tree, language.highlight);

        self.append_empty_lines(&mut tree_hl_blocks);
        Self::fill_in_empty_lines(&mut tree_hl_blocks);

        tree_hl_blocks
    }
}

#[derive(Debug, Clone)]
pub enum HLEnd {
    Bounded(usize),
    EndOfLine,
}

#[derive(Debug, Clone)]
pub struct HLBlock {
    pub start: usize,
    pub end: HLEnd,
    pub fg_color: Color,
    pub bg_color: Color,
}

impl<'a> HLBlock {
    pub const fn empty() -> Self {
        Self {
            start: 0,
            end: HLEnd::EndOfLine,
            fg_color: Color::DarkGrey,
            bg_color: Color::Reset,
        }
    }

    pub fn get_end(&self) -> Option<usize> {
        match self.end {
            HLEnd::EndOfLine => None,
            HLEnd::Bounded(end) => Some(end),
        }
    }

    pub fn get_end_unchecked(&self) -> usize {
        match self.end {
            HLEnd::EndOfLine => panic!("Called unchecked function on wrong variant"),
            HLEnd::Bounded(end) => end,
        }
    }

    pub fn slice_text(&self, line: &'a str) -> &'a str {
        match self.end {
            HLEnd::Bounded(end) => &line[self.start..end],
            HLEnd::EndOfLine => &line[self.start..],
        }
    }
}

fn hl_group_from_node(
    node: Node,
    hl_blocks: &mut Vec<Vec<HLBlock>>,
    node_type_to_color: fn(&str, &str, &str) -> Option<Color>,
) {
    let Some(parent) = node.parent() else { return };
    let last_sibiling_type = match node.prev_sibling() {
        Some(node) => node.kind(),
        None => "",
    };

    // Not every node needs a HLBlock, some are non-lexical nodes
    // Will return if on a non-lexical node
    let Some(color) = node_type_to_color(node.kind(), parent.kind(), last_sibiling_type) else {
        return;
    };

    let start: Point = node.start_position();
    let end: Point = node.end_position();

    if start.row != end.row {
        handle_new_line(start, end, color, hl_blocks);
    } else {
        add_block_to_row(
            start.row,
            start.column,
            HLEnd::Bounded(end.column),
            color,
            hl_blocks,
        );
    }
}

fn handle_new_line(start: Point, end: Point, color: Color, hl_blocks: &mut Vec<Vec<HLBlock>>) {
    assert!(start.row < end.row);

    // Goes to the end of the old line
    add_block_to_row(start.row, start.column, HLEnd::EndOfLine, color, hl_blocks);

    // Goes to the end of the in-between lines
    for line_idx in start.row + 1..end.row {
        add_block_to_row(line_idx, 0, HLEnd::EndOfLine, color, hl_blocks);
    }

    // Gets placed at the beginning of the last new line
    add_block_to_row(end.row, 0, HLEnd::Bounded(end.column), color, hl_blocks);
}

/// Assumes that `start.row == end.row`
/// The length of the row not including the new  
fn add_block_to_row(
    row: usize,
    mut start_column: usize,
    mut end_column: HLEnd,
    color: Color,
    hl_blocks: &mut Vec<Vec<HLBlock>>,
) {
    if row + 1 >= hl_blocks.len() {
        for _ in hl_blocks.len()..=row {
            hl_blocks.push(Vec::new());
        }
    }

    try_expand_hl_block_back(row, &mut start_column, &mut end_column, hl_blocks);

    let block = HLBlock {
        start: start_column,
        end: end_column,
        fg_color: color,
        bg_color: Color::Reset,
    };

    hl_blocks[row].push(block);
}

fn try_expand_hl_block_back(
    row: usize,
    start_column: &mut usize,
    end_column: &mut HLEnd,
    hl_blocks: &[Vec<HLBlock>],
) {
    if row >= hl_blocks.len() {
        return;
    }
    let row = &hl_blocks[row];

    match row.last() {
        // If this isn't the first block in the row, make sure there's no gap between the blocks
        Some(last_hl) => {
            if let Some(end) = last_hl.get_end() {
                *start_column = end;
            } else {
                // panic!("heeses");
            }
        }
        // If it is the first block, `start_column` should always be 0
        None => {
            *start_column = 0;

            // *end_column = 0;
        }
    }
}

fn highlight_tree(
    tree: &Tree,
    node_type_to_color: fn(&str, &str, &str) -> Option<Color>,
) -> Vec<Vec<HLBlock>> {
    let node_action = |cursor: &TreeCursor, hl_blocks: &mut Vec<Vec<HLBlock>>| {
        hl_group_from_node(cursor.node(), hl_blocks, node_type_to_color);
    };

    let move_up_hook = |_hl_blocks: &mut Vec<Vec<HLBlock>>| {};
    let move_down_hook = |_hl_blocks: &mut Vec<Vec<HLBlock>>| {};

    traverse_tree(tree, node_action, move_up_hook, move_down_hook)
}

// NOTE
// My special orange since my colorscheme (everforest) isn't actually base16 compliant
pub const ORANGE: Color = Color::Rgb {
    r: 230,
    g: 152,
    b: 117,
};
