use crate::highlight::ORANGE;
use crossterm::style::Color;

pub fn md_node_to_color(
    node_type: &str,
    _parent_type: &str,
    last_sibiling_type: &str,
) -> Option<Color> {
    let color = match node_type {
        "inline" => match last_sibiling_type {
            "atx_h1_marker" => Color::Red,
            "atx_h2_marker" => ORANGE,
            "atx_h3_marker" => Color::Yellow,
            "atx_h4_marker" => Color::Green,
            "atx_h5_marker" => Color::Blue,
            "atx_h6_marker" => Color::Magenta,

            _ => return None,
        },
        "paragraph" => Color::Grey,

        "atx_h1_marker" => Color::Red,
        "atx_h2_marker" => ORANGE,
        "atx_h3_marker" => Color::Yellow,
        "atx_h4_marker" | "link_text" => Color::Green,
        "atx_h5_marker" | "link_destination" | "list_marker_minus" => Color::Blue,
        "atx_h6_marker" => Color::Magenta,

        _ => return None,
    };

    Some(color)
}
