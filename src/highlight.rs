use std::cell::RefCell;

use crossterm::style::Color;
use tree_sitter::{Node, Parser, Point, Tree};

use crate::{DEBUG, buffer::Buffer, log, utility::is_symbol};

pub fn parse(buffer: &Buffer, parser: &RefCell<Parser>) -> Option<Tree> {
    let source: Vec<u8> = buffer.rope.bytes().collect();
    parser.borrow_mut().parse(source, None)
}

pub fn highlight(buffer: &Buffer, parser: Option<&RefCell<Parser>>) -> Vec<Vec<HLBlock>> {
    if let Some(tree) = parse(buffer, parser.unwrap()) {
        let mut tree_hl_blocks = highlight_tree(&tree);

        log!("Tree HL Blocks:\n\t{:?}\n", tree_hl_blocks);

        // Append lines without hl_block lists
        let buffer_lines = buffer.rope.len_lines();
        let hl_lines = tree_hl_blocks.len();
        if hl_lines < buffer_lines {
            for line_idx in hl_lines..buffer_lines {
                let line = buffer.rope.get_line(line_idx).unwrap();
                let block = HLBlock {
                    start: 0,
                    end: line.len_chars(),
                    color: Color::White,
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
                    end: line.len_chars(),
                    color: Color::White,
                };

                hl_blocks.push(block);
            });

        return tree_hl_blocks;
    }

    let hl_block = HLBlock {
        start: 0,
        end: buffer.len() - 1,
        color: Color::White,
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
}

fn hl_group_from_node<'a>(node: Node<'a>, hl_blocks: &mut Vec<Vec<HLBlock>>) {
    let node_type = node.kind();

    // Not every node needs a HLBlock, some are non-lexical nodes
    if let Some(color) = node_type_to_color(node_type) {
        let mut start: Point = node.start_position();
        let end: Point = node.end_position();

        // WARNING:
        // This assumes highlight groups are one line or less for now
        assert_eq!(start.row, end.row);

        if start.row + 1 >= hl_blocks.len() {
            for _ in hl_blocks.len()..start.row + 1 {
                hl_blocks.push(Vec::new());
            }
        }

        // Expand blocks backwards to consme un-highlighted sections
        if let Some(last_hl) = hl_blocks[start.row].last() {
            start.column = last_hl.end;
        }

        let block = HLBlock {
            start: start.column,
            end: end.column,
            color,
        };
        // log!("Block: {:?}\n", block);
        hl_blocks[start.row].push(block);
    }
}

fn highlight_tree(tree: &Tree) -> Vec<Vec<HLBlock>> {
    let mut hl_blocks: Vec<Vec<HLBlock>> = Vec::with_capacity(10);
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

fn node_type_to_color(node_type: &str) -> Option<Color> {
    let color = match node_type {
        "string_literal" => Color::Green,
        "identifier" => Color::White,
        "primitive_type" => Color::DarkMagenta,
        "number_literal" => Color::Magenta,
        ";" => Color::DarkGrey,
        str if str.chars().all(|c| is_symbol(c)) => Color::Yellow,
        _ => return None,
    };

    Some(color)
}
