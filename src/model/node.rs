use crate::model::*;
use colored::{ColoredString, Colorize};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::rc::Rc;
use termtree::Tree;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct CommitHash {
    full_hash: String,
}

impl Display for CommitHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.get_short_hash())
    }
}

impl CommitHash {
    pub fn new<S: Into<String>>(full_hash: S) -> Self {
        let full = full_hash.into();
        if full.len() < 8 {
            panic!("Commit hash must be at least 8 characters long");
        }
        CommitHash { full_hash: full }
    }
    pub fn get_full_hash(&self) -> &String {
        &self.full_hash
    }
    pub fn get_short_hash(&self) -> String {
        self.full_hash[0..8].to_string()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct CommitTag {
    tag: String,
    full_path: String,
}

impl CommitTag {
    pub fn new<S: Into<String>>(full_path: S) -> Self {
        let full_path = full_path.into();
        let normalized = full_path.to_normalized_path();
        let tag = normalized.last().unwrap().to_string();
        CommitTag { tag, full_path }
    }
    pub fn get_full_path(&self) -> &String {
        &self.full_path
    }
    pub fn get_tag(&self) -> &String {
        &self.tag
    }
}

#[derive(Clone, Debug)]
pub struct BranchData {
    branch: Option<String>,
    head: Option<CommitHash>,
}
impl BranchData {
    pub fn new(branch: Option<String>, head: Option<CommitHash>) -> Self {
        Self { branch, head }
    }
    pub fn empty() -> Self {
        Self {
            branch: None,
            head: None,
        }
    }
    pub fn has_branch(&self) -> bool {
        self.branch.is_some()
    }
    pub fn get_branch(&self) -> Option<&String> {
        self.branch.as_ref()
    }
    pub fn get_head(&self) -> Option<&CommitHash> {
        self.head.as_ref()
    }
}

pub enum PayloadType {
    Branch(BranchData),
    Tag(CommitTag),
}

#[derive(Debug)]
pub struct Node {
    name: String,
    node_type: NodeType,
    branch_data: BranchData,
    tags: Vec<CommitTag>,
    children: RefCell<HashMap<String, Rc<RefCell<Node>>>>,
}

impl Node {
    pub fn new<S: Into<String>>(
        name: S,
        node_type: NodeType,
        branch_data: BranchData,
        tags: Vec<CommitTag>,
    ) -> Self {
        Self {
            name: name.into(),
            node_type,
            branch_data,
            tags,
            children: RefCell::new(HashMap::new()),
        }
    }
    pub fn update_branch_data(&mut self, metadata: BranchData) {
        self.branch_data = metadata;
    }
    pub fn add_tag(&mut self, tag: CommitTag) {
        self.tags.push(tag);
    }
    pub fn update_type(&mut self, node_type: NodeType) {
        self.node_type = node_type;
    }
    fn build_display_tree(&self, show_tags: bool) -> Tree<String> {
        let mut formatted = ColoredString::from(self.name.clone());
        if self.branch_data.has_branch() {
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
                child.borrow_mut().update_branch_data(branch);
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
        &self.branch_data
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
    pub fn as_qualified_path(&self) -> NormalizedPath {
        NormalizedPath::from(self.name.clone())
    }
    pub fn display_tree(&self, show_tags: bool) -> String {
        self.build_display_tree(show_tags).to_string()
    }
}
