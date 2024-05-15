use std::ops::RangeBounds;

use database::Database;

use crate::Error;

use super::{node_db::NodeDB, Node, Range, Tree};

/// QueryTree is a "checked out" Tree at a given height which
/// borrows a Tree's NodeDb
#[derive(Debug)]
pub struct QueryTree<DB> {
    pub(crate) root: Option<Box<Node>>,
    pub(crate) node_db: NodeDB<DB>,
}

impl<DB: Database + Clone> QueryTree<DB> {
    pub fn new(tree: &Tree<DB>, mut version: u32) -> Result<Self, Error> {
        if version == 0 {
            version = tree.loaded_version;
        }

        if tree.versions.contains(&version) {
            let root = tree.node_db.get_root_node(version).expect(
                "the requested version is in the list of versions so the node should be in the db",
            );

            Ok(QueryTree {
                root,
                node_db: tree.node_db.clone(),
            })
        } else {
            Err(Error::VersionNotFound(version))
        }
    }
}

impl<DB: Database> QueryTree<DB> {
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        match &self.root {
            Some(root) => self.get_(key, root),
            None => None,
        }
    }

    // TODO: can we share this function with a regular tree's get_ method?
    fn get_(&self, key: &[u8], root: &Node) -> Option<Vec<u8>> {
        let mut loop_node = root;
        let mut cached_node;

        loop {
            match loop_node {
                Node::Leaf(leaf) => {
                    if leaf.key == key {
                        return Some(leaf.value.clone());
                    } else {
                        return None;
                    }
                }
                Node::Inner(node) => {
                    if key < &node.key {
                        match &node.left_node {
                            Some(left_node) => loop_node = left_node,
                            None => {
                                let left_node = self
                                    .node_db
                                    .get_node(&node.left_hash)
                                    .expect("node db should contain all nodes");

                                cached_node = left_node;
                                loop_node = &cached_node;
                            }
                        }
                    } else {
                        match &node.right_node {
                            Some(right_node) => loop_node = right_node,
                            None => {
                                let right_node = self
                                    .node_db
                                    .get_node(&node.right_hash)
                                    .expect("node db should contain all nodes");

                                cached_node = right_node;
                                loop_node = &cached_node;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn range<R>(&self, range: R) -> Range<'_, R, DB>
    where
        R: RangeBounds<Vec<u8>>,
    {
        match &self.root {
            Some(root) => Range {
                range,
                delayed_nodes: vec![root.clone()], //TODO: remove clone
                node_db: &self.node_db,
            },
            None => Range {
                range,
                delayed_nodes: vec![],
                node_db: &self.node_db,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use database::MemDB;

    #[test]
    fn new_query_tree_works() {
        let db = MemDB::new();
        let mut tree = Tree::new(db, None, 100.try_into().unwrap()).unwrap();
        tree.set(b"alice".to_vec(), b"abc".to_vec());
        tree.save_version().unwrap();
        tree.set(b"alice".to_vec(), b"123".to_vec());

        let query_tree = QueryTree::new(&tree, 1).unwrap();
        let result = query_tree.get(b"alice".as_slice()).unwrap();

        let expected = b"abc".to_vec();
        assert_eq!(expected, result);

        let result = tree.get(b"alice".as_slice()).unwrap();
        let expected = b"123".to_vec();
        assert_eq!(expected, result);
    }

    #[test]
    fn new_query_tree_works_empty_tree() {
        let db = MemDB::new();
        let mut tree = Tree::new(db, None, 100.try_into().unwrap()).unwrap();
        tree.save_version().unwrap();

        let query_tree = QueryTree::new(&tree, 1).unwrap();
        let result = query_tree.get(b"alice".as_slice());

        let expected = None;
        assert_eq!(expected, result);
    }
}
