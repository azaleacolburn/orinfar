use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

use crate::buffer::Buffer;

pub fn highlight(buffer: &Buffer) -> Vec<HighlightEvent> {
    let mut highlighter = Highlighter::new();
    let config =
        HighlightConfiguration::new(tree_sitter_c::LANGUAGE.into(), "c_highlight", "", "", "")
            .unwrap();
    let bytes = buffer.rope.bytes().collect::<Vec<u8>>();

    highlighter
        .highlight(&config, &bytes, None, |_| None)
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
}
