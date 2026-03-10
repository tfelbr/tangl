use crate::model::{FEATURES_PREFIX, PRODUCTS_PREFIX};
use colored::{ColoredString, Colorize};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub trait ValidNodeType: Clone + Debug {
    fn is_compatible(node_type: &NodeType) -> bool {
        Self::is_compatible_to_node_type(node_type)
    }
    fn is_compatible_to_node_type(node_type: &NodeType) -> bool;
}
pub trait CanHaveBranch: ValidNodeType {}

#[derive(Debug, Clone)]
pub struct WrongNodeTypeError {
    msg: String,
}
impl WrongNodeTypeError {
    pub fn new<S: Into<String>>(msg: S) -> WrongNodeTypeError {
        WrongNodeTypeError { msg: msg.into() }
    }
}
impl Display for WrongNodeTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl Error for WrongNodeTypeError {}

#[derive(Clone, Debug)]
pub struct Feature;
impl ValidNodeType for Feature {
    fn is_compatible_to_node_type(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::Feature => true,
            _ => false,
        }
    }
}
impl CanHaveBranch for Feature {}

#[derive(Clone, Debug)]
pub struct FeatureRoot;
impl ValidNodeType for FeatureRoot {
    fn is_compatible_to_node_type(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::FeatureRoot => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Product;
impl ValidNodeType for Product {
    fn is_compatible_to_node_type(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::Product => true,
            _ => false,
        }
    }
}
impl CanHaveBranch for Product {}

#[derive(Clone, Debug)]
pub struct ProductRoot;
impl ValidNodeType for ProductRoot {
    fn is_compatible_to_node_type(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::ProductRoot => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Area;
impl ValidNodeType for Area {
    fn is_compatible_to_node_type(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::Area => true,
            _ => false,
        }
    }
}
impl CanHaveBranch for Area {}

#[derive(Clone, Debug)]
pub struct VirtualRoot;
impl ValidNodeType for VirtualRoot {
    fn is_compatible_to_node_type(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::VirtualRoot => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tag;
impl ValidNodeType for Tag {
    fn is_compatible_to_node_type(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::Tag => true,
            _ => false,
        }
    }
}
impl CanHaveBranch for Tag {}

#[derive(Clone, Debug)]
pub struct AnyNode;
impl ValidNodeType for AnyNode {
    fn is_compatible_to_node_type(_node_type: &NodeType) -> bool {
        true
    }
}

#[derive(Clone, Debug)]
pub struct BranchAble;
impl ValidNodeType for BranchAble {
    fn is_compatible_to_node_type(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::Feature | NodeType::Product | NodeType::Area => true,
            _ => false,
        }
    }
}
impl CanHaveBranch for BranchAble {}

#[derive(Clone, Debug)]
pub enum NodeType {
    Feature,
    Product,
    FeatureRoot,
    ProductRoot,
    Area,
    VirtualRoot,
    Tag,
}

impl NodeType {
    pub fn build_child_from_name(&mut self, name: &str) -> Result<NodeType, WrongNodeTypeError> {
        match self {
            Self::Feature => Ok(Self::Feature),
            Self::Product => Ok(Self::Product),
            Self::FeatureRoot => Ok(Self::Feature),
            Self::ProductRoot => Ok(Self::Product),
            Self::VirtualRoot => Ok(Self::Area),
            Self::Area => {
                if name.starts_with(FEATURES_PREFIX) {
                    Ok(Self::FeatureRoot)
                } else if name.starts_with(PRODUCTS_PREFIX) {
                    Ok(Self::ProductRoot)
                } else {
                    Err(WrongNodeTypeError::new(format!(
                        "'{}' is no valid child of an area node. Valid childs include: feature, product",
                        name
                    )))
                }
            }
            Self::Tag => Err(WrongNodeTypeError::new("Tags cannot have children")),
        }
    }

    pub fn format_node_display(&self, name: ColoredString) -> ColoredString {
        match self {
            Self::FeatureRoot => name.bright_purple().bold(),
            Self::ProductRoot => name.red().bold(),
            Self::Tag => name.green(),
            _ => name,
        }
    }
}
