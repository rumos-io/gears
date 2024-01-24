pub mod node;
pub mod range;
pub mod query;

use crate::merkle::EMPTY_HASH;
use std::{
    cmp::Ordering,
    mem::{self},
};

use self::{node::Node, range::Range};

use super::HASH_LENGHT;

// TODO: rename loaded_version to head_version introduce a working_version (+ remove redundant loaded_version?). this will allow the first committed version to be version 0 rather than 1 (there is no version 0 currently!)
#[derive(Debug, Default)]
pub struct AvlTree {
    pub(crate) root: Option<Box<Node>>,
}

impl AvlTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_node(root: Node) -> Self {
        Self {
            root: Some(Box::new(root)),
        }
    }

    pub fn root_hash(&self) -> [u8; 32] {
        match &self.root {
            Some(root) => root.hash,
            None => EMPTY_HASH,
        }
    }

    pub fn get(&self, key: impl AsRef<[u8]>) -> Option<&Vec<u8>> {
        let mut current_tree = &self.root;

        while let Some(current_node) = current_tree {
            match current_node.key[..].cmp(key.as_ref()) {
                Ordering::Less => {
                    current_tree = &current_node.right_node;
                }
                Ordering::Equal => {
                    return Some(&current_node.value);
                }
                Ordering::Greater => {
                    current_tree = &current_node.left_node;
                }
            };
        }

        None
    }

    pub fn set(&mut self, key: &impl AsRef<[u8]>, value: Vec<u8>) -> Option<Vec<u8>> {
        let mut current_tree = &mut self.root;

        while let Some(current_node) = current_tree {
            match current_node.key[..].cmp(key.as_ref()) {
                Ordering::Less => {
                    current_tree = &mut current_node.right_node;
                }
                Ordering::Equal => {
                    let res = mem::replace(&mut current_node.value, value);

                    return Some(res);
                }
                Ordering::Greater => {
                    current_tree = &mut current_node.left_node;
                }
            };
        }

        None
    }

    pub fn insert(&mut self, node: Node) -> bool {
        let node = Box::new(node);
        return tree_insert(&mut self.root, node);

        fn tree_insert(tree: &mut Option<Box< Node >>, node: Box<Node>) -> bool {
            match tree {
                None => {
                    *tree = Some(node);
                    true
                },
                Some(tree_node) => {
                    let inserted =
                        match node.value.cmp(&node.value) {
                            Ordering::Equal => false,
                            Ordering::Less => tree_insert(&mut tree_node.right_node, node),
                            Ordering::Greater => tree_insert(&mut tree_node.left_node, node),
                        };
                    if inserted {
                        tree_node.update_height();
                        tree_node.update_size();
                        tree_node.rebalance();
                    }
                    inserted
                },
            }
        }
    }

    pub fn take(&mut self, value: &impl AsRef<[u8]>) -> Option<Node> {
        // Deref value and get copy on stack 
        return tree_take(&mut self.root, value).map( | this | *this );

        fn tree_take(tree: &mut Option<Box<Node>>, value: &impl AsRef<[u8]>) -> Option<Box<Node>> {
            match tree {
                None => return None,
                Some(node) => {
                    if let Some(result) =
                        match node.value.as_slice().cmp(value.as_ref()) {
                            Ordering::Less => Some(tree_take(&mut node.right_node, value)),
                            Ordering::Equal => None,
                            Ordering::Greater => Some(tree_take(&mut node.left_node, value)),
                        }
                    {
                        node.update_height();
                        node.rebalance();
                        return result
                    }
                },
            }
            // If control flow fell through to here, it's because we hit the Equal case above. The
            // borrow of `tree` is now out of scope, but we know it's Some node whose value is
            // equal to `value`. We can `take()` it out of the tree to get ownership of it, and
            // then we can manipulate the node and insert parts of it back into the tree as needed.

            let mut node = tree.take().expect( "We know that is some");
            match node.left_node.take() {
                None => {
                    *tree = node.right_node.take();
                },
                Some(left) => {
                    match node.right_node.take() {
                        None => {
                            *tree = Some(left);
                        },
                        Some(right) => {
                            // This is the general case: the node to be removed has both a left and
                            // a right child.
                            let mut replacement = leftmost_to_top(right);
                            replacement.left_node = Some(left);
                            replacement.update_height();
                            replacement.update_size();
                            replacement.rebalance();
                            *tree = Some(replacement);
                        }
                    }
                }
            }
            Some(node)
        }

        /// Returns a rotated version of `node` whose top has no left child and whose top has a
        /// balanced right subtree.
        fn leftmost_to_top(mut node: Box<Node>) -> Box<Node> {
            match node.left_node {
                None => node,
                Some(node_l) => {
                    let mut next_top = leftmost_to_top(node_l);
                    // By induction, next_top has no left child
                    node.left_node = next_top.right_node;
                    node.update_height();
                    node.rebalance();
                    next_top.right_node = Some(node);
                    next_top
                }
            }
        }
    }

    pub fn range<R>(&self, range: R) -> Range<R>
    where
        R: std::ops::RangeBounds<Vec<u8>>,
    {
        match &self.root {
            Some(root) => Range {
                range,
                delayed_nodes: vec![(**root).clone()], //TODO: remove clone
            },
            None => Range {
                range,
                delayed_nodes: vec![],
            },
        }
    }

    pub fn contains(&self, hash: &[u8; HASH_LENGHT]) -> bool {
        let mut current_tree = &self.root;

        while let Some(current_node) = current_tree {
            match current_node.hash.cmp(hash) {
                Ordering::Less => {
                    current_tree = &current_node.right_node;
                }
                Ordering::Equal => {
                    return true;
                }
                Ordering::Greater => {
                    current_tree = &current_node.left_node;
                }
            };
        }

        false
    }

    pub fn clear(&mut self) {
        self.root.take();
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }
}

#[cfg(test)]
mod tests {
    //TODO
}
