use crate::model::*;
use colored::Colorize;
use std::error::Error;
use std::rc::Rc;

pub const FEATURES_PREFIX: &str = "feature";
pub const PRODUCTS_PREFIX: &str = "product";

#[derive(Clone, Debug)]
pub struct TreeDataModel {
    virtual_root: Rc<Node>,
    qualified_paths_with_branch: Vec<QualifiedPath>,
}
impl TreeDataModel {
    pub fn new() -> Self {
        Self {
            virtual_root: Rc::new(Node::new(
                "",
                NodeType::VirtualRoot,
                NodeMetadata::default(),
            )),
            qualified_paths_with_branch: vec![],
        }
    }
    pub fn insert_qualified_path(
        &mut self,
        path: QualifiedPath,
        is_tag: bool,
    ) -> Result<(), WrongNodeTypeError> {
        if !path.is_absolute() {
            panic!("To insert a path, it must be absolute");
        }
        Rc::get_mut(&mut self.virtual_root)
            .unwrap()
            .insert_node_path(&path.strip_n_left(1), NodeMetadata::new(true), is_tag)?;
        self.qualified_paths_with_branch.push(path);
        Ok(())
    }
    pub fn get_area(&self, path: &QualifiedPath) -> Option<NodePath<Area>> {
        self.get_virtual_root().to_area(path)
    }
    pub fn get_virtual_root(&self) -> NodePath<VirtualRoot> {
        NodePath::<VirtualRoot>::new(vec![self.virtual_root.clone()])
    }
    pub fn get_node_path(&self, path: &QualifiedPath) -> Option<NodePath<AnyNode>> {
        let initial_path = self.get_virtual_root();
        let new_path = path.strip_n_left(1);
        initial_path.move_to(&new_path)
    }
    pub fn has_branch(&self, qualified_path: &QualifiedPath) -> bool {
        self.qualified_paths_with_branch
            .iter()
            .find(|e| *e == qualified_path)
            .is_some()
    }
    pub fn get_qualified_paths_with_branches(&self) -> &Vec<QualifiedPath> {
        &self.qualified_paths_with_branch
    }
    pub fn assert_all<T: SymbolicNodeType>(
        &self,
        paths: &Vec<QualifiedPath>,
    ) -> Result<Vec<NodePath<T>>, Box<dyn Error>> {
        let mut final_paths: Vec<NodePath<T>> = vec![];
        for path in paths.iter() {
            if let Some(path) = self.get_node_path(path) {
                if let Some(f) = path.try_convert_to::<T>() {
                    final_paths.push(f);
                } else {
                    return Err(format!(
                        "Path {} is not of type {}",
                        path.to_string().red(),
                        T::identifier()
                    )
                    .into());
                }
            } else {
                return Err(format!(
                    "Path {} does not exist in current working tree",
                    path.to_string().red()
                )
                .into());
            }
        }
        Ok(final_paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_node_path_with_virtual_root() {
        let mut tree = TreeDataModel::new();
        tree.insert_qualified_path(QualifiedPath::from("/main"), false)
            .unwrap();
        let path = tree.get_node_path(&QualifiedPath::from("/main")).unwrap();
        assert_eq!(path.to_qualified_path(), "/main")
    }
}
