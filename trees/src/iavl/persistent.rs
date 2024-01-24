use std::collections::BTreeSet;

use database::Database;

use crate::{merkle::EMPTY_HASH, Error};

use super::{accessor::Accessor, node::Node, range::Range, AvlTree};

// TODO: rename loaded_version to head_version introduce a working_version (+ remove redundant loaded_version?).
// this will allow the first committed version to be version 0 rather than 1 (there is no version 0 currently!)
#[derive(Debug)]
pub struct Tree<T> {
    pub inner_tree: AvlTree,
    pub(crate) accessor: Accessor<T, Node>,

    /// Renamed from `loaded_version`
    pub(crate) head_version: u32,
    // TODO:
    #[allow(dead_code)]
    pub(crate) working_version: u32,
    pub(crate) versions: BTreeSet<u32>,
}

impl<T: Database> Tree<T> {
    /// Creates new `Self`
    ///
    /// # Panics
    /// Panics if `cache_size = 0`
    pub fn new(db: T, target_version: Option<u32>, cache_size: usize) -> Result<Tree<T>, Error> {
        assert!(cache_size > 0);
        let node_db = Accessor::new(db, cache_size);
        let versions = node_db.get_versions();

        if let Some(target_version) = target_version {
            let root = node_db.get_root_node(target_version)?;

            Ok(Tree {
                inner_tree: match root {
                    Some(root) => AvlTree::with_node(root),
                    None => AvlTree::new(),
                },
                accessor: node_db,
                head_version: target_version,
                working_version: target_version,
                versions,
            })
        } else {
            // use the latest version available
            if let Some(latest_version) = versions.last() {
                Ok(Tree {
                    inner_tree: match node_db
                        .get_root_node(*latest_version)
                        .expect("invalid data in database - possible database corruption")
                    {
                        Some(root) => AvlTree::with_node(root),
                        None => AvlTree::new(),
                    },
                    accessor: node_db,
                    head_version: *latest_version,
                    working_version: *latest_version,
                    versions,
                })
            } else {
                Ok(Tree {
                    inner_tree: AvlTree::new(),
                    accessor: node_db,
                    head_version: 0,
                    working_version: 0,
                    versions,
                })
            }
        }
    }

    pub fn loaded_version(&self) -> u32 {
        self.head_version
    }

    /// Save the current tree to disk.
    /// Returns an error if saving would overwrite an existing version
    pub fn save_version(&mut self) -> Result<([u8; 32], u32), Error> {
        let version = self.head_version + 1;

        if self.versions.contains(&version) {
            // If the version already exists, return an error as we're attempting to overwrite.
            // However, the same hash means idempotent (i.e. no-op).
            // TODO: do we really need to be doing this?
            let saved_hash = self.accessor.get_root_hash(version)?;
            let working_hash = self.inner_tree.root_hash();

            if saved_hash == working_hash {
                self.head_version = version;

                // clear the root node's left and right nodes if they exist
                if let Some(node) = &mut self.inner_tree.root {
                    let _ = node.left_node.take();
                    let _ = node.right_node.take();
                }
                return Ok((saved_hash, self.head_version));
            }
            return Err(Error::Overwrite);
        }

        let root_hash = self.save_tree(version);
        self.versions.insert(version);

        self.head_version = version;
        Ok((root_hash, self.head_version))
    }

    fn recursive_tree_save(accessor: &mut Accessor<T, Node>, node: &Node) {
        if let Some(left_node) = &node.left_node {
            Self::recursive_tree_save(accessor, left_node);
        }
        if let Some(right_node) = &node.right_node {
            Self::recursive_tree_save(accessor, right_node);
        }

        accessor.save_node(node)
    }

    /// Saves the given node and all of its descendants.
    /// Clears left_node/right_node on the root.
    pub(crate) fn save_tree(&mut self, version: u32) -> [u8; 32] {
        if let Some(root) = &mut self.inner_tree.root {
            Self::recursive_tree_save(&mut self.accessor, root);

            let _ = root.left_node.take();
            let _ = root.right_node.take();

            self.accessor.save_version(version, &root.hash);

            root.hash
        } else {
            self.accessor.save_version(version, &EMPTY_HASH);
            EMPTY_HASH
        }
    }

    pub fn set(&mut self, key: &impl AsRef<[u8]>, value: Vec<u8>) -> Option<Vec<u8>> {
        self.inner_tree.set(key, value)
    }

    pub fn range<R>(&self, range: R) -> Range<R>
    where
        R: std::ops::RangeBounds<Vec<u8>>,
    {
        self.inner_tree.range(range)
    }

    pub fn get(&self, key: impl AsRef<[u8]>) -> Option<&Vec<u8>> {
        self.inner_tree.get(key)
    }
}
