use crate::view_box::ViewBox;
use crossterm::style::Color;
use tree_sitter::{Node, Point, Tree};

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
    node_type_to_color: fn(&str, &str) -> Option<Color>,
) {
    let node_type = node.kind();

    let Some(parent) = node.parent() else { return };

    // Will return if on a non-lexical node
    let Some(color) = node_type_to_color(node_type, parent.kind()) else {
        return;
    };

    // Not every node needs a HLBlock, some are non-lexical nodes
    let start: Point = node.start_position();
    let end: Point = node.end_position();

    // WARNING:
    // This assumes highlight groups are one line or less for now
    // assert_eq!(start.row, end.row);

    if start.row != end.row {
        assert!(start.row < end.row);

        let first_end = Point {
            row: start.row,
            column: 0, // NOTE Will actually go to the end of the line
        };

        add_block_to_row(start, first_end, color, hl_blocks, true);

        for line_idx in start.row + 1..end.row {
            let middle_start = Point {
                row: line_idx,
                column: 0,
            };

            let middle_end = Point {
                row: line_idx,
                column: 0, // NOTE Will actually go to the end of the line
            };

            add_block_to_row(middle_start, middle_end, color, hl_blocks, true);
        }

        let last_start = Point {
            row: end.row,
            column: 0,
        };

        add_block_to_row(last_start, end, color, hl_blocks, false);

        return;
    }

    add_block_to_row(start, end, color, hl_blocks, false);
}

/// Assumes that `start.row == end.row`
/// The length of the row not including the new  
fn add_block_to_row(
    mut start: Point,
    end: Point,
    color: Color,
    hl_blocks: &mut Vec<Vec<HLBlock>>,
    to_end_of_line: bool,
) {
    assert_eq!(start.row, end.row);

    if start.row + 1 >= hl_blocks.len() {
        for _ in hl_blocks.len()..=start.row {
            hl_blocks.push(Vec::new());
        }
    }

    // Expand blocks backwards to consume un-highlighted sections
    if let Some(last_hl) = hl_blocks[start.row].last() {
        // NOTE
        // You cannot add a hl block to a row where the last hl block goes to the end
        // start.column = match last_hl.end {
        //     HLEnd::Bounded(n) => n,
        //     HLEnd::EndOfLine => {last_hl.end = HLEnd::Bounded(line_len);},
        // }
        start.column = last_hl.get_end_unchecked();
    } else if end.column != 0 && hl_blocks[start.row].is_empty() {
        // Expand block backwards if there's whitespace or other non-parsable content at the
        // beginning of the line
        start.column = 0;
    }

    let end = if to_end_of_line {
        HLEnd::EndOfLine
    } else {
        HLEnd::Bounded(end.column)
    };

    let block = HLBlock {
        start: start.column,
        end,
        fg_color: color,
        bg_color: Color::Reset,
    };
    hl_blocks[start.row].push(block);
}

fn highlight_tree(
    tree: &Tree,
    node_type_to_color: fn(&str, &str) -> Option<Color>,
) -> Vec<Vec<HLBlock>> {
    let mut hl_blocks: Vec<Vec<HLBlock>> = Vec::new();
    let mut cursor = tree.walk();

    // Depth-first search with a cursor instead of recursion
    'TREE_WALK: loop {
        hl_group_from_node(cursor.node(), &mut hl_blocks, node_type_to_color);

        if cursor.goto_first_child() {
            continue;
        }

        if cursor.goto_next_sibling() {
            continue;
        }

        'BACKTRACKING: loop {
            // If we don't have a parent to go to, we're at the top of the tree and done traversing
            if !cursor.goto_parent() {
                break 'TREE_WALK;
            }

            // If we have a sibiling, we're done backtracking for now
            if cursor.goto_next_sibling() {
                break 'BACKTRACKING;
            }
        }
    }

    hl_blocks
}

// NOTE
// My special orange since my colorscheme (everforest) isn't actually base16 compliant
pub const ORANGE: Color = Color::Rgb {
    r: 230,
    g: 152,
    b: 117,
};
