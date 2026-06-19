use crate::DEBUG;
use crossterm::style::Color;

pub fn md_node_to_color(node_type: &str, _parent_type: &str) -> Option<Color> {
    log!("{}", node_type);
    let color = match node_type {
        "text" => Color::Grey,
        "list_marker" => Color::Red,
        "atx_h2_marker" => Color::Red,
        "inline" => Color::Grey,
        "paragraph" => Color::Grey,

        _ => return None,
    };

    Some(color)
}
