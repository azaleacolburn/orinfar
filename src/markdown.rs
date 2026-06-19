use crate::DEBUG;
use crossterm::style::Color;

pub fn md_node_to_color(node_type: &str, _parent_type: &str) -> Option<Color> {
    log!("{}", node_type);
    let color = match node_type {
        "text" => Color::Grey,
        "inline" => Color::Grey,
        "paragraph" => Color::Grey,

        "atx_h2_marker" => Color::Red,

        "list_marker" => Color::Red,

        _ => return None,
    };

    Some(color)
}
