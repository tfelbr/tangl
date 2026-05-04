use crate::core::model::node::*;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

pub trait SymbolicNodeType: Clone + Debug + Eq + PartialEq + Hash {
    fn identifier() -> String;
    fn is_compatible(node: &Node) -> bool {
        Self::is_compatible_to_node(node)
    }
    fn is_compatible_to_node(node: &Node) -> bool;
}

pub trait BranchType: Clone + Debug + Eq + PartialEq + Hash {}

pub trait HasFeatureChildren: SymbolicNodeType {}
pub trait HasProductChildren: SymbolicNodeType {}
pub trait IsGitObject: SymbolicNodeType {}
pub trait IsOnOrUnderArea: SymbolicNodeType {}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Branch;
impl BranchType for Branch {}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct NoBranch;
impl BranchType for NoBranch {}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AnyBranchType;
impl BranchType for AnyBranchType {}

/*
    Feature Definition
*/

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Feature<T: BranchType> {
    _phantom: PhantomData<T>,
}
impl<T: BranchType> SymbolicNodeType for Feature<T> {
    fn identifier() -> String {
        NodeType::ConcreteFeature.get_type_name()
    }

    fn is_compatible_to_node(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::ConcreteFeature => true,
            _ => false,
        }
    }
}
impl<T: BranchType> HasFeatureChildren for Feature<T> {}
impl<T: BranchType> IsOnOrUnderArea for Feature<T> {}
impl IsGitObject for Feature<Branch> {}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct FeatureRoot;
impl SymbolicNodeType for FeatureRoot {
    fn identifier() -> String {
        NodeType::FeatureRoot.get_type_name()
    }

    fn is_compatible_to_node(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::FeatureRoot => true,
            _ => false,
        }
    }
}
impl HasFeatureChildren for FeatureRoot {}
impl IsOnOrUnderArea for FeatureRoot {}

/*
   Product Definition
*/

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Product<T: BranchType> {
    _phantom: PhantomData<T>,
}
impl<T: BranchType> SymbolicNodeType for Product<T> {
    fn identifier() -> String {
        NodeType::ConcreteProduct.get_type_name()
    }

    fn is_compatible_to_node(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::ConcreteProduct => true,
            _ => false,
        }
    }
}
impl<T: BranchType> HasProductChildren for Product<T> {}
impl<T: BranchType> IsOnOrUnderArea for Product<T> {}
impl IsGitObject for Product<Branch> {}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ProductRoot;
impl SymbolicNodeType for ProductRoot {
    fn identifier() -> String {
        NodeType::ProductRoot.get_type_name()
    }

    fn is_compatible_to_node(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::ProductRoot => true,
            _ => false,
        }
    }
}
impl HasProductChildren for ProductRoot {}
impl IsOnOrUnderArea for ProductRoot {}

/*
    Area Definition
*/

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Area<T: BranchType> {
    _phantom: PhantomData<T>,
}
impl<T: BranchType> SymbolicNodeType for Area<T> {
    fn identifier() -> String {
        NodeType::Area.get_type_name()
    }

    fn is_compatible_to_node(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::Area => true,
            _ => false,
        }
    }
}
impl<T: BranchType> IsOnOrUnderArea for Area<T> {}
impl IsGitObject for Area<Branch> {}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Temporary;
impl SymbolicNodeType for Temporary {
    fn identifier() -> String {
        NodeType::Temporary.get_type_name()
    }

    fn is_compatible_to_node(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::Temporary => true,
            _ => false,
        }
    }
}
impl IsGitObject for Temporary {}
impl IsOnOrUnderArea for Temporary {}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct VirtualRoot;
impl SymbolicNodeType for VirtualRoot {
    fn identifier() -> String {
        NodeType::VirtualRoot.get_type_name()
    }

    fn is_compatible_to_node(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::VirtualRoot => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AnyNode;
impl SymbolicNodeType for AnyNode {
    fn identifier() -> String {
        "any".to_string()
    }

    fn is_compatible_to_node(_node_type: &NodeType) -> bool {
        true
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AnyGitObject;
impl SymbolicNodeType for AnyGitObject {
    fn identifier() -> String {
        "git object".to_string()
    }

    fn is_compatible_to_node(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::ConcreteFeature
            | NodeType::ConcreteProduct
            | NodeType::Area
            | NodeType::Temporary => true,
            _ => false,
        }
    }
}
impl IsGitObject for AnyGitObject {}
