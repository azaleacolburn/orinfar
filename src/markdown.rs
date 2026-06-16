use crossterm::style::Color;

pub fn md_node_to_color(node_type: &str, parent_type: &str) -> Option<Color> {
    let color = match node_type {
        "text" => Color::Grey,
        "list_marker" => Color::Red,

        _ => return None,
    };

    Some(color)
}
