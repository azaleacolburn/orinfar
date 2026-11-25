use crate::view_box::{RenderInfo, ViewBox};
use anyhow::Result;

#[derive(PartialEq, Clone)]
pub enum ViewNodeDirection {
    LeftOrTop,
    RightOrBottom,
}

impl ViewNodeDirection {
    pub fn other(&self) -> ViewNodeDirection {
        if *self == ViewNodeDirection::LeftOrTop {
            ViewNodeDirection::RightOrBottom
        } else {
            ViewNodeDirection::LeftOrTop
        }
    }
}

pub type ViewNodePath = Vec<ViewNodeDirection>;
pub type ViewNodePathSlice<'a> = &'a [ViewNodeDirection];

#[derive(PartialEq)]
pub enum ViewNode {
    Leaf(Box<ViewBox>),
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
    pub fn evaluate_view_node_path(&self, path: ViewNodePathSlice) -> Option<&ViewNode> {
        let Some((direction, remaining_path)) = path.split_first() else {
            // `path` was empty
            return Some(self);
        };

        match self {
            ViewNode::Leaf(_) => {
                log!("Invalid View Node Path");

                None
            }
            ViewNode::SplitVertical { top, bottom } => match direction {
                ViewNodeDirection::LeftOrTop => top.evaluate_view_node_path(remaining_path),
                ViewNodeDirection::RightOrBottom => bottom.evaluate_view_node_path(remaining_path),
            },
            ViewNode::SplitHorizontal { left, right } => match direction {
                ViewNodeDirection::LeftOrTop => left.evaluate_view_node_path(remaining_path),
                ViewNodeDirection::RightOrBottom => right.evaluate_view_node_path(remaining_path),
            },
        }
    }

    pub fn evaluate_view_box_path(&self, path: ViewNodePathSlice) -> Option<&ViewBox> {
        match self.evaluate_view_node_path(path) {
            Some(ViewNode::Leaf(b)) => Some(b),
            _ => None,
        }
    }

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
                log!("leaf");
                view_box.render(
                    RenderInfo {
                        x,
                        y,
                        height,
                        width,
                    },
                    adjusted,
                    resized,
                )?;
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
