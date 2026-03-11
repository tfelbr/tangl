use crate::model::*;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::rc::Rc;

pub trait NodePathBasicNavigation
where
    Self: Sized,
{
    fn move_to(self, path: &QualifiedPath) -> Option<NodePath<AnyNode>>;
    fn move_to_last_valid(self, path: &QualifiedPath) -> NodePath<AnyNode>;
}
pub trait NodePathFeatureNavigation: NodePathBasicNavigation
where
    Self: Sized,
{
    fn move_to_feature(self, path: &QualifiedPath) -> Option<NodePath<Feature>> {
        self.move_to(path)?.try_as_concrete_type()
    }
}
pub trait NodePathProductNavigation: NodePathBasicNavigation
where
    Self: Sized,
{
    fn move_to_product(self, path: &QualifiedPath) -> Option<NodePath<Product>> {
        self.move_to(path)?.try_as_concrete_type()
    }
}

pub enum ConcreteNodePathType {
    Feature(NodePath<Feature>),
    FeatureRoot(NodePath<FeatureRoot>),
    Product(NodePath<Product>),
    ProductRoot(NodePath<ProductRoot>),
    Area(NodePath<Area>),
    VirtualRoot(NodePath<VirtualRoot>),
    Tag(NodePath<Tag>),
}

#[derive(Clone, Debug)]
pub struct NodePath<T: ValidNodeType> {
    path: Vec<Rc<Node>>,
    _phantom: PhantomData<T>,
}

impl<T: CanHaveBranch> NodePath<T> {
    pub fn to_git_branch(&self) -> String {
        self.to_qualified_path().to_git_branch()
    }
    pub fn as_branch_able(&self) -> NodePath<BranchAble> {
        NodePath::new(self.path.clone())
    }
}

impl NodePath<AnyNode> {
    fn as_concrete_type<T: ValidNodeType>(&self) -> NodePath<T> {
        NodePath::<T>::new(self.path.clone())
    }
    pub fn try_as_concrete_type<T: ValidNodeType>(&self) -> Option<NodePath<T>> {
        if T::is_compatible(self.get_node().get_type()) {
            Some(self.as_concrete_type())
        } else {
            None
        }
    }
    pub fn from_concrete<T: ValidNodeType>(other: &NodePath<T>) -> Self {
        Self::new(other.path.clone())
    }
}

impl NodePath<VirtualRoot> {
    pub fn to_area(self, area: &QualifiedPath) -> Option<NodePath<Area>> {
        self.move_to(area)?.try_as_concrete_type()
    }
}

impl NodePath<Area> {
    pub fn get_path_to_feature_root(&self) -> QualifiedPath {
        self.to_qualified_path() + QualifiedPath::from(FEATURES_PREFIX)
    }
    pub fn get_path_to_product_root(&self) -> QualifiedPath {
        self.to_qualified_path() + QualifiedPath::from(PRODUCTS_PREFIX)
    }
    pub fn move_to_feature_root(self) -> Option<NodePath<FeatureRoot>> {
        self.move_to(&QualifiedPath::from(FEATURES_PREFIX))?
            .try_as_concrete_type()
    }
    pub fn move_to_product_root(self) -> Option<NodePath<ProductRoot>> {
        self.move_to(&QualifiedPath::from(PRODUCTS_PREFIX))?
            .try_as_concrete_type()
    }
}

impl NodePath<FeatureRoot> {
    pub fn iter_root_features(&self) -> impl Iterator<Item = NodePath<Feature>> {
        self.iter_children()
            .map(|p| p.try_as_concrete_type().unwrap())
    }
    pub fn iter_features_req(&self) -> impl Iterator<Item = NodePath<Feature>> {
        self.iter_children_req()
            .map(|p| p.try_as_concrete_type().unwrap())
    }
}

impl NodePathProductNavigation for NodePath<ProductRoot> {}
impl NodePathProductNavigation for NodePath<Product> {}

impl NodePathFeatureNavigation for NodePath<FeatureRoot> {}
impl NodePathFeatureNavigation for NodePath<Feature> {}

impl<T: ValidNodeType> ToQualifiedPath for NodePath<T> {
    fn to_qualified_path(&self) -> QualifiedPath {
        let mut path = QualifiedPath::new();
        for p in self.path.iter() {
            path.push(p.get_name());
        }
        path
    }
}

impl<T: ValidNodeType> NodePath<T> {
    fn get_node(&self) -> &Node {
        self.path.last().unwrap()
    }
    pub fn new(path: Vec<Rc<Node>>) -> NodePath<T> {
        Self {
            path,
            _phantom: PhantomData,
        }
    }
    pub fn iter_children(&self) -> impl Iterator<Item = NodePath<AnyNode>> {
        self.get_node().iter_children().map(|(name, _)| {
            self.clone()
                .move_to(&QualifiedPath::from(name.clone()))
                .unwrap()
        })
    }
    pub fn iter_children_req(&self) -> impl Iterator<Item = NodePath<AnyNode>> {
        self.iter_children().flat_map(|path| {
            let mut to_iter = Vec::new();
            to_iter.push(path.clone());
            to_iter.extend(path.iter_children_req());
            to_iter
        })
    }
    pub fn get_tags(&self) -> Vec<QualifiedPath> {
        self.get_node()
            .iter_children()
            .filter_map(|(name, child)| match child.get_type() {
                NodeType::Tag => Some(QualifiedPath::from(name.clone())),
                _ => None,
            })
            .collect()
    }
    pub fn get_metadata(&self) -> &NodeMetadata {
        self.get_node().get_metadata()
    }
    pub fn as_any_type(&self) -> NodePath<AnyNode> {
        NodePath::<AnyNode>::from_concrete(self)
    }
    pub fn display_tree(&self, show_tags: bool) -> String {
        self.get_node().display_tree(show_tags)
    }
}

impl<T: ValidNodeType> NodePathBasicNavigation for NodePath<T> {
    fn move_to(mut self, path: &QualifiedPath) -> Option<NodePath<AnyNode>> {
        for p in path.iter_string() {
            self.path.push(self.get_node().get_child(p)?.clone());
        }
        Some(NodePath::<AnyNode>::new(self.path))
    }

    fn move_to_last_valid(self, path: &QualifiedPath) -> NodePath<AnyNode> {
        let mut current = self.as_any_type();
        for part in path.iter() {
            let next = current.clone().move_to(&part);
            if next.is_some() {
                current = next.unwrap();
            } else {
                break;
            }
        }
        current
    }
}

impl<A, B> PartialEq<NodePath<A>> for NodePath<B>
where A: ValidNodeType,
B: ValidNodeType,
{
    fn eq(&self, other: &NodePath<A>) -> bool {
        self.to_qualified_path() == other.to_qualified_path()
    }
}

impl<T: ValidNodeType> Eq for NodePath<T> {}

impl<T: ValidNodeType> Display for NodePath<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_qualified_path().to_string().as_str())
    }
}
