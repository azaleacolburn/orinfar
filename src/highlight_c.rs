use std::cell::RefCell;

use crossterm::style::Color;
use tree_sitter::{Node, Parser, Point, Tree};

use crate::{DEBUG, buffer::Buffer, log, utility::is_symbol};

pub fn parse(buffer: &Buffer, parser: &RefCell<Parser>) -> Option<Tree> {
    let source: Vec<u8> = buffer.rope.bytes().collect();
    parser.borrow_mut().parse(source, None)
}

pub fn highlight(buffer: &Buffer, parser: &RefCell<Parser>) -> Vec<Vec<HLBlock>> {
    if let Some(tree) = parse(buffer, parser) {
        let mut tree_hl_blocks = highlight_tree(&tree);

        // Chop multi-line hl blocks

        log!("Tree HL Blocks:\n\t{:?}\n", tree_hl_blocks);

        // Append lines without hl_block lists
        let buffer_lines = buffer.rope.len_lines();
        let hl_lines = tree_hl_blocks.len();
        if hl_lines < buffer_lines {
            for line_idx in hl_lines..buffer_lines {
                let line = buffer.rope.get_line(line_idx).unwrap();
                let block = HLBlock {
                    start: 0,
                    end: if line_idx + 1 == buffer_lines {
                        line.len_chars()
                    } else {
                        line.len_chars() - 1
                    },
                    color: Color::Grey,
                    to_end_of_line: false,
                };
                tree_hl_blocks.push(vec![block])
            }
        }

        // Fill in empty lines
        tree_hl_blocks
            .iter_mut()
            .enumerate()
            .filter(|(_, l)| l.is_empty())
            .for_each(|(line_idx, hl_blocks)| {
                let line = buffer.rope.get_line(line_idx).unwrap();
                let block = HLBlock {
                    start: 0,
                    end: if line_idx + 1 == buffer_lines {
                        line.len_chars()
                    } else {
                        line.len_chars() - 1
                    },
                    color: Color::Grey,
                    to_end_of_line: false,
                };

                hl_blocks.push(block);
            });

        return tree_hl_blocks;
    }

    let hl_block = HLBlock {
        start: 0,
        end: buffer.len() - 1,
        to_end_of_line: false, // Technically true, but not needed
        color: Color::Grey,
    };

    return (0..buffer.rope.len_lines())
        .map(|_| vec![hl_block.clone()])
        .collect();
}

#[derive(Debug, Clone)]
pub struct HLBlock {
    pub start: usize,
    pub end: usize,
    pub color: crossterm::style::Color,
    // If this is set, ignore `self.end` and make the block go to the end of the line
    pub to_end_of_line: bool,
}

fn hl_group_from_node<'a>(node: Node<'a>, hl_blocks: &mut Vec<Vec<HLBlock>>) {
    let node_type = node.kind();

    let parent = match node.parent() {
        Some(p) => p,
        None => return,
    };

    // Will return if on a non-lexical node
    let color = match node_type_to_color(node_type, parent.kind()) {
        Some(c) => c,
        None => return,
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
        for _ in hl_blocks.len()..start.row + 1 {
            hl_blocks.push(Vec::new());
        }
    }

    // Expand blocks backwards to consme un-highlighted sections
    if let Some(last_hl) = hl_blocks[start.row].last() {
        start.column = last_hl.end;
    } else if end.column != 0 && hl_blocks[start.row].len() == 0 {
        // Expand block backwards if there's whitespace or other non-parsable content at the
        // beginning of the line
        start.column = 0;
    }

    let block = HLBlock {
        start: start.column,
        end: end.column,
        to_end_of_line,
        color,
    };
    // log!("Block: {:?}\n", block);
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
    "for", "while", "if", "continue", "break", "return", "asm", "register", "extern",
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
    log!("node_type: {}", node_type);
    let color = match node_type {
        "#include" => Color::DarkRed,

        "#define" => Color::DarkRed,
        "preproc_arg" => Color::Grey,

        "#ifdef" => Color::DarkRed,
        "#ifndef" => Color::DarkRed,
        "#endif" => Color::DarkRed,

        "string_content" | "character" | "\"" | "\'" | "system_lib_string" => Color::Green,

        "identifier"
            if parent_type == "function_declarator" || parent_type == "call_expression" =>
        {
            Color::Green
        }
        "identifier" => Color::Grey,
        "field_identifier" => Color::Blue,

        "primitive_type" => Color::Yellow,
        "type_identifier" => Color::DarkMagenta,

        "number_literal" => Color::Magenta,
        "comment" => Color::DarkGrey,
        ";" | "." | "," => Color::DarkGrey,
        str if is_operator(str) => ORANGE,
        str if str.chars().all(|c| is_symbol(c)) => Color::Grey,
        str if is_c_keyword(str) => Color::Red,
        _ => return None,
    };

    Some(color)
}
