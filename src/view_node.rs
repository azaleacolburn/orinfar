use crate::view_box::ViewBox;

pub enum ViewNodeDirection {
    Left,
    Right,
}

pub type ViewNodePath = Vec<ViewNodeDirection>;

pub enum ViewNode {
    Leaf(Box<ViewBox>),
    SplitVertical {
        left: Box<ViewNode>,
        right: Box<ViewNode>,
    },
    SplitHorizontal {
        top: Box<ViewNode>,
        bottom: Box<ViewNode>,
    },
}
