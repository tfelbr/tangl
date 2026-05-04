use crate::core::model::commit::CommitTag;
use crate::core::model::*;
use colored::{ColoredString, Colorize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::rc::Rc;
use termtree::Tree;
use thiserror::Error;

pub const FEATURE_ROOT: &str = "feature";
pub const PRODUCT_ROOT: &str = "product";
pub const TEMPORARY: &str = "tmp";

#[derive(Error, Debug)]
pub struct WrongNodeTypeError {
    types_expected: Vec<NodeType>,
    type_found: NodeType,
}

impl WrongNodeTypeError {
    pub fn new(types_expected: Vec<NodeType>, type_found: NodeType) -> Self {
        Self {
            types_expected,
            type_found,
        }
    }
}

impl Display for WrongNodeTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub enum PayloadType {
    Branch(String),
    Tag(CommitTag),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum NodeType {
    VirtualRoot,
    Area(bool),
    FeatureRoot,
    ProductRoot,
    Feature(bool),
    Product(bool),
    Temporary(bool),
    Unknown,
}

impl NodeType {
    pub fn decide_next_type(&self, name: &str, branch: bool) -> NodeType {
        match self {
            Self::VirtualRoot => Self::Area(branch),
            Self::Area(_) => match name {
                FEATURE_ROOT => Self::FeatureRoot,
                PRODUCT_ROOT => Self::ProductRoot,
                TEMPORARY => Self::Temporary(branch),
                _ => Self::Unknown,
            },
            Self::Feature(_) | Self::FeatureRoot => Self::Feature(branch),
            Self::Product(_) | Self::ProductRoot => Self::Product(branch),
            Self::Temporary(_) => Self::Temporary(branch),
            Self::Unknown => Self::Unknown,
        }
    }

    pub fn format_node_display(&self, name: ColoredString) -> ColoredString {
        match self {
            Self::Area(_) => name.yellow(),
            Self::FeatureRoot => name.bright_purple(),
            Self::Feature(_) => name.purple(),
            Self::ProductRoot => name.truecolor(231, 100, 18),
            Self::Product(_) => name.truecolor(231, 100, 18),
            _ => name,
        }
    }

    pub fn get_type_name(&self) -> String {
        let name: &str = match self {
            Self::VirtualRoot => "virtual root",
            Self::Area => "area",
            Self::FeatureRoot => "feature root",
            Self::ProductRoot => "product root",
            Self::Feature => "feature",
            Self::Product => "product",
            Self::Temporary => "temporary",
            Self::Unknown => "",
        };
        name.to_string()
    }

    pub fn get_short_type_name(&self) -> String {
        let name: &str = match self {
            Self::VirtualRoot => "vr",
            Self::Area => "a",
            Self::FeatureRoot => "fr",
            Self::ProductRoot => "pr",
            Self::Feature => "f",
            Self::Product => "p",
            Self::Temporary => "temp",
            Self::Unknown => "",
        };
        name.to_string()
    }

    pub fn get_formatted_name(&self) -> String {
        self.format_node_display(self.get_type_name().normal())
            .to_string()
    }

    pub fn get_formatted_short_name(&self) -> String {
        self.format_node_display(self.get_short_type_name().normal())
            .to_string()
    }
}

#[derive(Debug)]
pub struct Node {
    name: String,
    node_type: NodeType,
    branch: Option<String>,
    tags: Vec<CommitTag>,
    children: RefCell<HashMap<String, Rc<RefCell<Node>>>>,
}

impl Node {
    pub fn new (
        name: String,
        node_type: NodeType,
        branch: Option<String>,
        tags: Vec<CommitTag>,
    ) -> Self {
        Self {
            name,
            node_type,
            branch,
            tags,
            children: RefCell::new(HashMap::new()),
        }
    }
    fn update_type(&mut self, node_type: NodeType) {
        self.node_type = node_type;
    }
    fn update_branch(&mut self, branch: Option<String>) {
        self.branch = branch;
    }
    fn build_display_tree(&self, show_tags: bool) -> Tree<String> {
        let mut formatted = ColoredString::from(self.name.clone());
        if self.branch.has_branch() {
            formatted = formatted.blue()
        }
        let type_display = match self.node_type {
            NodeType::AbstractFeature | NodeType::AbstractProduct => None,
            _ => Some(self.node_type.get_formatted_short_name()),
        };
        let content = if let Some(type_display) = type_display {
            format!("{formatted} [{type_display}]")
        } else {
            formatted.to_string()
        };
        let mut tree = Tree::<String>::new(content);
        let children = self.children.borrow();
        let mut sorted_children = children.iter().collect::<Vec<_>>();
        sorted_children.sort_by(|a, b| b.0.chars().cmp(a.0.chars()));
        sorted_children.reverse();
        for (_, child) in sorted_children {
            tree.leaves
                .push(child.borrow().build_display_tree(show_tags));
        }
        tree
    }
    fn decide_child_type<S: Into<String>>(&self, name: S, metadata: &BranchData) -> NodeType {
        let real_name = name.into();
        self.node_type
            .decide_next_type(real_name.as_str(), metadata)
    }
    fn add_child<S: Into<String>>(&self, name: S, metadata: PayloadType) -> NodeType {
        let real_name = name.into();
        let (branch, tags) = match metadata {
            PayloadType::Branch(branch) => (branch, vec![]),
            PayloadType::Tag(tag) => {
                let branch = BranchData::empty();
                (branch, vec![tag])
            }
        };
        let node_type = self.decide_child_type(real_name.clone(), &branch);
        let child = Rc::new(RefCell::new(Node::new(
            real_name.clone(),
            node_type.clone(),
            branch,
            tags,
        )));
        self.children.borrow_mut().insert(real_name, child);
        node_type
    }
    fn update_child<S: Into<String>>(&self, name: S, metadata: PayloadType) -> NodeType {
        let real_name = name.into();
        let node_type = match metadata {
            PayloadType::Branch(branch) => {
                let new_type = self.decide_child_type(real_name.clone(), &branch);
                let child = self.get_child(real_name).unwrap();
                child.borrow_mut().update_type(new_type.clone());
                child.borrow_mut().update_branch(branch);
                new_type
            }
            PayloadType::Tag(tag) => {
                let child = self.get_child(real_name.clone()).unwrap();
                child.borrow_mut().add_tag(tag);
                child.borrow().get_type().clone()
            }
        };
        node_type
    }
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_type(&self) -> &NodeType {
        &self.node_type
    }
    pub fn get_branch_data(&self) -> &BranchData {
        &self.branch
    }
    pub fn get_tags(&self) -> &Vec<CommitTag> {
        &self.tags
    }
    pub fn get_child<S: Into<String>>(&self, name: S) -> Option<Rc<RefCell<Node>>> {
        Some(self.children.borrow().get(&name.into())?.clone())
    }
    pub fn has_children(&self) -> bool {
        !self.children.borrow().is_empty()
    }
    pub fn get_children(&self) -> Vec<Rc<RefCell<Node>>> {
        let nodes = self.children.borrow();
        nodes.values().cloned().collect()
    }
    pub fn insert_path(&self, path: &NormalizedPath, metadata: PayloadType) -> NodeType {
        match path.len() {
            0 => self.node_type.clone(),
            1 => {
                let name = path.get(0).unwrap().to_string();
                let new_type = match self.get_child(&name) {
                    Some(_) => self.update_child(name, metadata),
                    None => self.add_child(name.clone(), metadata),
                };
                new_type
            }
            _ => {
                let name = path.get(0).unwrap().to_string();
                let next_child = match self.get_child(&name) {
                    Some(node) => node,
                    None => {
                        self.add_child(name.clone(), PayloadType::Branch(BranchData::empty()));
                        self.get_child(&name).unwrap()
                    }
                };
                next_child
                    .borrow_mut()
                    .insert_path(&path.strip_n_left(1), metadata)
            }
        }
    }
    pub fn add_tag(&mut self, tag: CommitTag) {
        self.tags.push(tag);
    }
    pub fn display_tree(&self, show_tags: bool) -> String {
        self.build_display_tree(show_tags).to_string()
    }
}
