use crate::model::error::WrongNodeTypeError;
use crate::model::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct TreeDataModel {
    virtual_root: Rc<RefCell<Node>>,
    qualified_paths_with_branch: Vec<NormalizedPath>,
    unknowns_exist: RefCell<bool>,
}
impl TreeDataModel {
    pub fn new() -> Self {
        Self {
            virtual_root: Rc::new(RefCell::new(Node::new(
                "",
                NodeType::VirtualRoot,
                BranchData::empty(),
                vec![],
            ))),
            qualified_paths_with_branch: vec![],
            unknowns_exist: RefCell::new(false),
        }
    }
    pub fn insert_git_branch<S1: Into<String>, S2: Into<String>>(
        &self,
        path: S1,
        head: S2,
    ) -> NodeType {
        let branch = path.into();
        let normalized_path = branch.to_normalized_path();
        let hash = CommitHash::new(head);
        let branch_data = BranchData::new(Some(branch), Some(hash));
        let node_type = self
            .virtual_root
            .borrow_mut()
            .insert_path(&normalized_path, PayloadType::Branch(branch_data));
        match node_type {
            NodeType::Unknown => {
                self.unknowns_exist.replace(true);
            }
            _ => {}
        }
        node_type
    }
    pub fn insert_tag<S: Into<String>>(&self, path: S) {
        let path = path.into();
        let normalized_path = path.to_normalized_path();
        let tag = CommitTag::new(path);
        self.virtual_root.borrow_mut().insert_path(
            &normalized_path.strip_n_right(normalized_path.len() - 1),
            PayloadType::Tag(tag),
        );
    }
    pub fn get_area(&self, path: &NormalizedPath) -> Option<NodePath<ConcreteArea>> {
        self.get_virtual_root().move_to_area(path)
    }
    pub fn get_virtual_root(&self) -> NodePath<VirtualRoot> {
        NodePath::<VirtualRoot>::new(
            vec![self.virtual_root.clone()],
            self.unknowns_exist.borrow().clone(),
            PointsTo::Head,
        )
    }
    pub fn get_node_path<T: SymbolicNodeType>(&self, path: &NormalizedPath) -> Option<NodePath<T>> {
        let initial_path = self.get_virtual_root();
        let new_path = path.strip_n_left(1);
        initial_path.move_to(&new_path)?.try_convert_to()
    }
    pub fn has_branch(&self, qualified_path: &NormalizedPath) -> bool {
        self.qualified_paths_with_branch
            .iter()
            .find(|e| *e == qualified_path)
            .is_some()
    }
    pub fn get_qualified_paths_with_branches(&self) -> &Vec<NormalizedPath> {
        &self.qualified_paths_with_branch
    }
    pub fn assert_path<T: SymbolicNodeType>(
        &self,
        path: &NormalizedPath,
    ) -> Result<NodePath<T>, ModelError> {
        if let Some(node_path) = self.get_node_path::<AnyNode>(path) {
            if let Some(concrete) = node_path.try_convert_to::<T>() {
                Ok(concrete)
            } else {
                Err(WrongNodeTypeError::new(format!(
                    "NodeTypeError for {}: expected to be of type '{}', but is of type '{}'",
                    node_path,
                    T::identifier(),
                    node_path.get_actual_type().get_type_name()
                ))
                .into())
            }
        } else {
            Err(PathNotFoundError::new(format!("Path {} does not exist", path)).into())
        }
    }
    pub fn assert_all<T: SymbolicNodeType>(
        &self,
        paths: &Vec<NormalizedPath>,
    ) -> Result<Vec<NodePath<T>>, ModelError> {
        let mut final_paths: Vec<NodePath<T>> = vec![];
        for path in paths.iter() {
            final_paths.push(self.assert_path::<T>(path)?);
        }
        Ok(final_paths)
    }
}
