use crate::model::*;
use colored::{ColoredString, Colorize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use termtree::Tree;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct CommitHash {
    full_hash: String,
}

impl CommitHash {
    pub fn new<S: Into<String>>(full_hash: S) -> Self {
        CommitHash {
            full_hash: full_hash.into(),
        }
    }
    pub fn get_full_hash(&self) -> &String {
        &self.full_hash
    }
    pub fn get_short_hash(&self) -> String {
        self.full_hash[0..8].to_string()
    }
}


#[derive(Clone, Debug)]
pub struct NodeMetadata {
    branch: Option<String>,
    head: Option<CommitHash>,
}
impl NodeMetadata {
    pub fn new(branch: Option<String>, head: Option<CommitHash>) -> Self {
        Self { branch, head }
    }
    pub fn empty() -> Self {
        Self { branch: None, head: None }
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

#[derive(Clone, Debug)]
pub struct Node {
    name: String,
    node_type: NodeType,
    metadata: NodeMetadata,
    children: HashMap<String, Rc<Node>>,
}

impl Node {
    pub fn new<S: Into<String>>(name: S, node_type: NodeType, metadata: NodeMetadata) -> Self {
        Self {
            name: name.into(),
            node_type,
            metadata,
            children: HashMap::new(),
        }
    }
    pub fn update_metadata(&mut self, metadata: NodeMetadata) {
        self.metadata = metadata;
    }
    pub fn update_type(&mut self, node_type: NodeType) {
        self.node_type = node_type;
    }
    fn build_display_tree(&self, show_tags: bool) -> Tree<String> {
        let mut formatted = ColoredString::from(self.name.clone());
        if self.metadata.has_branch() {
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
        let mut sorted_children = self.children.iter().collect::<Vec<_>>();
        sorted_children.sort_by(|a, b| b.0.chars().cmp(a.0.chars()));
        sorted_children.reverse();
        for (_, child) in sorted_children {
            match child.node_type {
                NodeType::Tag => {
                    if !show_tags {
                        continue;
                    }
                }
                _ => {}
            }
            tree.leaves.push(child.build_display_tree(show_tags));
        }
        tree
    }
    fn decide_child_type<S: Into<String>>(
        &self,
        name: S,
        metadata: &NodeMetadata,
        is_tag: bool,
    ) -> NodeType {
        let real_name = name.into();
        let new_type = if is_tag {
            NodeType::Tag
        } else {
            self.node_type
                .decide_next_type(real_name.as_str(), metadata)
        };
        new_type
    }
    fn add_child<S: Into<String>>(
        &mut self,
        name: S,
        metadata: NodeMetadata,
        is_tag: bool,
    ) -> NodeType {
        let real_name = name.into();
        let new_type = self.decide_child_type(real_name.clone(), &metadata, is_tag);
        self.children.insert(
            real_name.clone(),
            Rc::new(Node::new(real_name, new_type.clone(), metadata)),
        );
        new_type
    }
    fn update_child<S: Into<String>>(
        &mut self,
        name: S,
        metadata: NodeMetadata,
        is_tag: bool,
    ) -> NodeType {
        let real_name = name.into();
        let new_type = self.decide_child_type(real_name.clone(), &metadata, is_tag);
        let child = self.get_child_mut(real_name).unwrap();
        child.update_metadata(metadata);
        child.update_type(new_type.clone());
        new_type
    }
    fn get_child_mut<S: Into<String>>(&mut self, name: S) -> Option<&mut Node> {
        let real_name = name.into();
        let maybe_mut = Rc::get_mut(self.children.get_mut(&real_name)?);
        match maybe_mut {
            Some(node) => Some(node),
            None => panic!(
                "Tried to get child '{}' as mutable but failed: shared references exist\n\
                Make sure to drop all references to the node tree if you attempt modifications",
                real_name
            ),
        }
    }
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_type(&self) -> &NodeType {
        &self.node_type
    }
    pub fn get_metadata(&self) -> &NodeMetadata {
        &self.metadata
    }
    pub fn get_child<S: Into<String>>(&self, name: S) -> Option<&Rc<Node>> {
        Some(self.children.get(&name.into())?)
    }
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
    pub fn iter_children(&self) -> impl Iterator<Item = (&String, &Rc<Node>)> {
        self.children.iter()
    }
    pub fn insert_node_path(
        &mut self,
        path: &NormalizedPath,
        metadata: NodeMetadata,
        is_tag: bool,
    ) -> NodeType {
        let name = path.get(0).unwrap().to_string();
        match path.len() {
            0 => self.node_type.clone(),
            1 => {
                let new_type = match self.get_child_mut(&name) {
                    Some(_) => self.update_child(name, metadata, is_tag),
                    None => self.add_child(name.clone(), metadata, is_tag),
                };
                new_type
            }
            _ => {
                let next_child = match self.get_child_mut(&name) {
                    Some(node) => node,
                    None => {
                        self.add_child(name.clone(), NodeMetadata::empty(), false);
                        self.get_child_mut(&name).unwrap()
                    }
                };
                next_child.insert_node_path(&path.strip_n_left(1), metadata, is_tag)
            }
        }
    }
    pub fn as_qualified_path(&self) -> NormalizedPath {
        NormalizedPath::from(self.name.clone())
    }
    pub fn get_qualified_paths_by<T, P>(
        &self,
        initial_path: &NormalizedPath,
        predicate: &P,
        categories: &Vec<T>,
    ) -> HashMap<T, Vec<NormalizedPath>>
    where
        P: Fn(&T, &Node) -> bool,
        T: Hash + Eq + Clone + Debug,
    {
        let mut result: HashMap<T, Vec<NormalizedPath>> = HashMap::new();
        for child in self.children.values() {
            let path = initial_path.clone() + child.as_qualified_path();
            for t in categories {
                let to_insert = if predicate(t, child) {
                    vec![path.clone()]
                } else {
                    vec![]
                };
                if result.contains_key(t) {
                    result.get_mut(t).unwrap().extend(to_insert);
                } else {
                    result.insert(t.clone(), to_insert);
                }
            }
            let from_child = child.get_qualified_paths_by(&path, predicate, categories);
            for (t, value) in from_child {
                result.get_mut(&t).unwrap().extend(value);
            }
        }
        result
    }
    pub fn display_tree(&self, show_tags: bool) -> String {
        self.build_display_tree(show_tags).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prepare_node() -> Node {
        let mut node = Node::new("root", NodeType::ConcreteFeature, NodeMetadata::empty());
        node.insert_node_path(
            &NormalizedPath::from("foo/f1"),
            NodeMetadata::empty(),
            false,
        );
        node.insert_node_path(
            &NormalizedPath::from("bar/b1"),
            NodeMetadata::empty(),
            false,
        );
        node
    }

    #[test]
    fn test_get_qualified_paths_by() {
        let predicate = |_: &i32, node: &Node| -> bool { !node.get_metadata().has_branch() };
        let node = prepare_node();
        let result = node
            .get_qualified_paths_by(&NormalizedPath::new(), &predicate, &vec![0])
            .get(&0)
            .unwrap()
            .clone();
        assert!(result.contains(&NormalizedPath::from("foo")));
        assert!(result.contains(&NormalizedPath::from("bar")));
        assert!(result.contains(&NormalizedPath::from("foo/f1")));
        assert!(result.contains(&NormalizedPath::from("bar/b1")));
    }
}
