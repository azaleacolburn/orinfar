use crate::view_box::{RenderInfo, ViewBox};
use anyhow::Result;

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
    pub fn render_view_node(
        &mut self,
        x: u16,
        y: u16,
        height: u16,
        width: u16,
        adjusted: bool,
        resized: bool,
    ) -> Result<()> {
        match self {
            ViewNode::Leaf(view_box) => {
                view_box.set_render_info(RenderInfo {
                    x,
                    y,
                    height,
                    width,
                });
                view_box.render(adjusted, resized)?;
            }

            ViewNode::SplitHorizontal { left, right } => {
                left.render_view_node(x, y, height, width / 2, adjusted, resized)?;

                let right_width = height.div_ceil(2);
                let right_x = x + (width - right_width);

                right.render_view_node(right_x, y, height, right_width, adjusted, resized)?;
            }

            ViewNode::SplitVertical { top, bottom } => {
                top.render_view_node(x, y, height / 2, width, adjusted, resized)?;

                let bottom_height = height.div_ceil(2);
                let bottom_y = y + (height - bottom_height);

                bottom.render_view_node(x, bottom_y, bottom_height, width, adjusted, resized)?;
            }
        }

        Ok(())
    }
}
