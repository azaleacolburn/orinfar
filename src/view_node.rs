use crate::view_box::ViewBox;

pub enum ViewNode {
    Leaf(ViewBox),
    SplitVertical {
        left: Box<ViewNode>,
        right: Box<ViewNode>,
    },
    SplitHorizontal {
        top: Box<ViewNode>,
        bottom: Box<ViewNode>,
    },
}
