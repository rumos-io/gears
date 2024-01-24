pub mod node;
pub mod range;
pub mod query;

use crate::merkle::EMPTY_HASH;
use std::{
    cmp::Ordering,
    mem::{self, replace},
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

    pub fn take(&mut self, hash: &[u8; HASH_LENGHT]) -> Option<Vec<u8>> {
        let mut prev_ptrs = Vec::<*mut Node>::new();
        let mut current_tree = &mut self.root;
        let mut target_value = None;

        while let Some(current_node) = current_tree {
            match current_node.hash.cmp(hash) {
                Ordering::Less => {
                    prev_ptrs.push(&mut **current_node);
                    current_tree = &mut current_node.right_node;
                }
                Ordering::Equal => {
                    target_value = Some(&mut **current_node);
                    break;
                }
                Ordering::Greater => {
                    prev_ptrs.push(&mut **current_node);
                    current_tree = &mut current_node.left_node;
                }
            };
        }

        if target_value.is_none() {
            return None;
        }

        let target_node = target_value.unwrap();

        let taken_value = if target_node.left_node.is_none() || target_node.right_node.is_none() {
            if let Some(left_node) = target_node.left_node.take() {
                mem::replace(target_node, *left_node).value
            } else if let Some(right_node) = target_node.right_node.take() {
                mem::replace(target_node, *right_node).value
            } else if let Some(prev_ptr) = prev_ptrs.pop() {
                let prev_node = unsafe { &mut *prev_ptr };

                let inner_value = if let Some(left_node) = prev_node.left_node.as_ref() {
                    if left_node.value == target_node.value {
                        prev_node.left_node.take().unwrap().value
                    } else {
                        prev_node.right_node.take().unwrap().value
                    }
                } else {
                    prev_node.right_node.take().unwrap().value
                };

                prev_node.update_height();
                prev_node.update_size();
                prev_node.rebalance();

                inner_value
            } else {
                self.root.take().unwrap().value
            }
        } else {
            let right_tree = &mut target_node.right_node;

            if right_tree.as_ref().unwrap().left_node.is_none() {
                let mut right_node = right_tree.take().unwrap();

                let inner_value = mem::replace(&mut target_node.value, right_node.value);
                let _ = mem::replace(&mut target_node.right_node, right_node.right_node.take());

                target_node.update_height();
                target_node.update_size();
                target_node.rebalance();

                inner_value
            } else {
                let mut next_tree = right_tree;
                let mut inner_ptrs = Vec::<*mut Node>::new();

                while let Some(next_left_node) = next_tree {
                    if next_left_node.left_node.is_some() {
                        inner_ptrs.push(&mut **next_left_node);
                    }
                    next_tree = &mut next_left_node.left_node;
                }

                let parent_left_node = unsafe { &mut *inner_ptrs.pop().unwrap() };
                let mut leftmost_node = parent_left_node.left_node.take().unwrap();

                let inner_value = mem::replace(&mut target_node.value, leftmost_node.value);
                let _ = replace(
                    &mut parent_left_node.left_node,
                    leftmost_node.right_node.take(),
                );

                parent_left_node.update_height();
                parent_left_node.update_size();
                parent_left_node.rebalance();

                for node_ptr in inner_ptrs.into_iter().rev() {
                    let node = unsafe { &mut *node_ptr };
                    node.update_height();
                    node.update_size();
                    node.rebalance();
                }

                target_node.update_height();
                target_node.update_size();
                target_node.rebalance();

                inner_value
            }
        };

        for node_ptr in prev_ptrs.into_iter().rev() {
            let node = unsafe { &mut *node_ptr };
            node.update_height();
            node.update_size();
            node.rebalance();
        }

        Some(taken_value)
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
