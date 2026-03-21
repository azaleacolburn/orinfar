use std::io::stdout;

use tree_sitter::{Parser, Tree};
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

use crate::buffer::Buffer;

const HIGHLIGHT_NAMES: &[&str] = &[
    "attribute",
    "comment",
    "constant",
    "constant.builtin",
    "constructor",
    "embedded",
    "function",
    "function.builtin",
    "keyword",
    "module",
    "number",
    "operator",
    "property",
    "property.builtin",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.special",
    "string",
    "string.special",
    "tag",
    "type",
    "type.builtin",
    "variable",
    "variable.builtin",
    "variable.parameter",
];

pub fn parse(buffer: &Buffer, parser: &mut Parser) -> Option<Tree> {
    let source: Vec<u8> = buffer.rope.bytes().collect();
    parser.parse(source, None)
}

pub fn highlight(buffer: &Buffer) -> Vec<HighlightEvent> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_c::LANGUAGE.into())
        .expect("Failed to load C parser");
    if let Some(tree) = parse(buffer, &mut parser) {
        let mut cursor = tree.walk();
        assert_eq!(cursor.field_name().unwrap(), "source_file");

        return vec![];
    } else {
        return vec![];
    }

    let mut config = HighlightConfiguration::new(
        tree_sitter_c::LANGUAGE.into(),
        tree_sitter_c::HIGHLIGHT_QUERY,
        "",
        "",
        "",
    )
    .unwrap();

    config.configure(HIGHLIGHT_NAMES);

    let bytes = buffer.rope.bytes().collect::<Vec<u8>>();

    let mut highlighter = Highlighter::new();
    highlighter
        .highlight(&config, b"int count = 0;", None, |_| None)
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
}
