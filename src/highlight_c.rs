use crossterm::style::Color;
use tree_sitter::{Node, Point, Tree};

use crate::{DEBUG, log, utility::is_symbol, view_box::ViewBox};

impl ViewBox {
    pub fn parse(&mut self) -> Option<&Tree> {
        if self.buffer.has_changed {
            let source: Vec<u8> = self.buffer.rope.bytes().collect();
            // TODO
            // Edit the `Tree` and pass it to the `parse` function
            self.parse_tree = self.parser.as_mut()?.parse(source, None);
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
    pub fn fill_in_empty_lines(tree_hl_blocks: &mut Vec<Vec<HLBlock>>) {
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
        if let Some(tree) = &self.parse_tree {
            let mut tree_hl_blocks = highlight_tree(tree);

            self.append_empty_lines(&mut tree_hl_blocks);
            Self::fill_in_empty_lines(&mut tree_hl_blocks);

            return tree_hl_blocks;
        }

        return vec![];
    }
}

#[derive(Debug, Clone)]
pub enum HLEnd {
    Bounded(usize),
    EndOfLine,
}

// TODO
// Turn this bad boy into an enum
// Variants: eol, bounded
#[derive(Debug, Clone)]
pub struct HLBlock {
    pub start: usize,
    pub end: HLEnd,
    pub color: crossterm::style::Color,
}

impl<'a> HLBlock {
    pub fn empty() -> Self {
        HLBlock {
            start: 0,
            end: HLEnd::EndOfLine,
            color: Color::DarkGrey,
        }
    }

    pub fn get_end(&self, line: &str) -> usize {
        match self.end {
            HLEnd::EndOfLine => line.len(),
            HLEnd::Bounded(end) => end,
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

fn hl_group_from_node(node: Node, hl_blocks: &mut Vec<Vec<HLBlock>>) {
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
fn add_block_to_row(
    mut start: Point,
    end: Point,
    color: Color,
    hl_blocks: &mut Vec<Vec<HLBlock>>,
    to_end_of_line: bool,
) {
    if start.row + 1 >= hl_blocks.len() {
        for _ in hl_blocks.len()..=start.row {
            hl_blocks.push(Vec::new());
        }
    }

    // Expand blocks backwards to consme un-highlighted sections
    if let Some(last_hl) = hl_blocks[start.row].last() {
        start.column = last_hl.get_end_unchecked();
    } else if end.column != 0 && hl_blocks[start.row].is_empty() {
        // Expand block backwards if there's whitespace or other non-parsable content at the
        // beginning of the line
        start.column = 0;
    }

    let block = HLBlock {
        start: start.column,
        end: HLEnd::Bounded(end.column),
        color,
    };
    hl_blocks[start.row].push(block);
}

fn highlight_tree(tree: &Tree) -> Vec<Vec<HLBlock>> {
    let mut hl_blocks: Vec<Vec<HLBlock>> = Vec::new();
    let mut cursor = tree.walk();

    // Depth-first search with a cursor instead of recursion
    'TREE_WALK: loop {
        hl_group_from_node(cursor.node(), &mut hl_blocks);

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

const KEYWORDS: &[&str] = &[
    "alignas",
    "alignof",
    "auto",
    "bool",
    "break",
    "case",
    "char",
    "const",
    "constexpr",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extern",
    "false",
    "float",
    "for",
    "goto",
    "if",
    "inline",
    "int",
    "long",
    "nullptr",
    "register",
    "restrict",
    "return",
    "short",
    "signed",
    "sizeof",
    "static",
    "static_assert",
    "struct",
    "switch",
    "thread_local",
    "true",
    "typedef",
    "typeof ",
    "typeof_unqual",
    "union",
    "unsigned",
    "void",
    "volatile",
    "while",
    // "_Alignof",
    // "_Atomic",
    // "_BitInt",
    // "_Bool",
    // "_Complex",
    // "_Decimal128",
    // "_Decimal32",
    // "_Decimal64",
    // "_Generic",
    // "_Imaginary",
    // "_Noreturn",
    // "_Static_assert",
    // "_Thread_local",
];

fn is_c_keyword(str: &str) -> bool {
    KEYWORDS.contains(&str)
}

const OPERATORS: &[&str] = &[
    "+", "-", "/", "*", "++", "--", "+=", "-=", "<", ">", "||", "&&", "<=", ">=", "||=", "&&=",
    "!", "!=", "==", "=", "&", "|", "^", "~", "<<", ">>", "|=", "&=", "~=", "^=", "->",
];

fn is_operator(str: &str) -> bool {
    OPERATORS.contains(&str)
}

// My special orange since my colorscheme (everforest) isn't actually base16 compliant
const ORANGE: Color = Color::Rgb {
    r: 230,
    g: 152,
    b: 117,
};

fn node_type_to_color(node_type: &str, parent_type: &str) -> Option<Color> {
    log!("node_type {}", node_type);
    let color = match node_type {
        "#include" | "#define" | "#ifdef" | "#ifndef" | "#endif" => Color::DarkRed,

        "string_content" | "character" | "\"" | "\'" | "system_lib_string" => Color::Green,
        "identifier"
            if parent_type == "function_declarator" || parent_type == "call_expression" =>
        {
            Color::Green
        }

        "identifier" | "preproc_arg" => Color::Grey,
        "field_identifier" => Color::Blue,

        "primitive_type" => Color::Yellow,
        "type_identifier" => Color::DarkMagenta,

        "number_literal" => Color::Magenta,
        "comment" | ";" | "." | "," => Color::DarkGrey,
        // Common macros
        // In the future a way to automatically determine
        // which strings are macros would be really cool
        "true" | "false" | "NULL" => Color::Magenta,
        str if is_operator(str) => ORANGE,
        str if str.chars().all(is_symbol) => Color::Grey,
        str if is_c_keyword(str) => Color::Red,
        _ => return None,
    };

    Some(color)
}
