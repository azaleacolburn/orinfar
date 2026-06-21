use crate::{c::c_node_to_color, markdown::md_node_to_color};
use crossterm::style::Color;

pub struct OrinLanguage {
    pub extensions: Vec<String>,
    pub highlight: fn(&str, &str, &str) -> Option<Color>,
    pub lang: tree_sitter::Language,
}

impl OrinLanguage {
    pub fn new<'a>(
        extensions: &'a [&'a str],
        lang: tree_sitter::Language,
        highlight: fn(&str, &str, &str) -> Option<Color>,
    ) -> Self {
        Self {
            extensions: extensions.iter().map(|s| (*s).to_string()).collect(),
            lang,
            highlight,
        }
    }

    pub fn from_ext(extension: &str) -> Option<Self> {
        let lang = match extension {
            "c" | "h" => Self::new(&["c", "h"], tree_sitter_c::LANGUAGE.into(), c_node_to_color),
            "md" => Self::new(&["md"], tree_sitter_md::LANGUAGE.into(), md_node_to_color),
            _ => {
                return None;
            }
        };

        Some(lang)
    }
}
