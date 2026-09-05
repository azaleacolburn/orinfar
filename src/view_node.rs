use crate::view_box::{RenderInfo, ViewBox};

pub enum ViewNodeDirection {
    Left,
    Right,
}

pub type ViewNodePath = Vec<ViewNodeDirection>;

pub enum ViewNode {
    Leaf(ViewBox),
    SplitHorizontal {
        left: Box<ViewNode>,
        right: Box<ViewNode>,
    },
    SplitVertical {
        top: Box<ViewNode>,
        bottom: Box<ViewNode>,
    },
}

impl ViewNode {
    /// Renders a view node by recursively rendering all of its children
    pub fn render_view_node(&self, x: u16, y: u16, height: u16, width: u16, adjusted: bool) {
        match self {
            ViewNode::Leaf(view_box) => {
                view_box.render(
                    RenderInfo {
                        x,
                        y,
                        height,
                        width,
                    },
                    adjusted,
                );
            }

            ViewNode::SplitHorizontal { left, right } => {
                left.render_view_node(x, y, height, width / 2, adjusted);

                let right_width = height.div_ceil(2);
                let right_x = x + (width - right_width);

                right.render_view_node(right_x, y, height, right_width, adjusted);
            }

            ViewNode::SplitVertical { top, bottom } => {
                top.render_view_node(x, y, height / 2, width, adjusted);

                let bottom_height = height.div_ceil(2);
                let bottom_y = y + (height - bottom_height);

                bottom.render_view_node(x, bottom_y, bottom_height, width, adjusted);
            }
        }
    }
}
