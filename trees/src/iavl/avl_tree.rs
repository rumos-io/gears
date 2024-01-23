use crate::{error::Error, merkle::EMPTY_HASH};
use integer_encoding::VarInt;
use std::{
    cmp::{self, Ordering},
    mem::{self, replace},
};

use super::HASH_LENGHT;

/// Value which stored in AVL Tree.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NodeValue {
    Unloaded,
    Bytes(Vec<u8>),
    // TODO: Ask Kevin about other types
}

impl From<Vec<u8>> for NodeValue {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(value)
    }
}

#[derive(Debug)]
pub struct Node {
    height: u8,
    size: u32, // number of nodes in this subtree. If it exceeds 2 than tree should be rotated
    version: u32,
    pub(crate) key: Vec<u8>,
    pub(crate) value: NodeValue,
    pub(crate) hash: [u8; HASH_LENGHT],
    pub(crate) left_node: Option<Box<Node>>,
    pub(crate) right_node: Option<Box<Node>>,
}

impl Node {
    /// Return hash of left node
    pub fn left_hash(&self) -> [u8; HASH_LENGHT] //TODO: Is it better Option or empty?
    {
        self.left_node.as_ref().map_or(EMPTY_HASH, |left| left.hash)
    }

    /// Return height of left node
    pub fn left_height(&self) -> u8 {
        self.left_node.as_ref().map_or(0, |left| left.height)
    }

    /// Return hash of right node
    pub fn right_hash(&self) -> [u8; HASH_LENGHT] //TODO: Is it better Option or empty?
    {
        self.right_node
            .as_ref()
            .map_or(EMPTY_HASH, |right| right.hash)
    }

    /// Return height of right node
    pub fn right_height(&self) -> u8 {
        self.right_node.as_ref().map_or(0, |right| right.height)
    }

    /// Update height of node taking max value from left or right
    pub fn update_height(&mut self) {
        self.height = 1 + cmp::max(self.left_height(), self.right_height());
    }

    pub fn balance_factor(&self) -> i8 {
        let left_height = self.left_height();
        let right_height = self.right_height();

        if left_height >= right_height {
            (left_height - right_height) as i8
        } else {
            -((right_height - left_height) as i8)
        }
    }

    pub fn rotate_left(&mut self) -> bool {
        if let Some(right_node) = &mut self.right_node {
            let right_left_tree = right_node.left_node.take();
            let right_right_tree = right_node.right_node.take();

            let mut new_left_tree = mem::replace(&mut self.right_node, right_right_tree);
            mem::swap(&mut self.value, &mut new_left_tree.as_mut().unwrap().value);
            let left_tree = self.left_node.take();

            let new_left_node = new_left_tree.as_mut().unwrap();
            new_left_node.right_node = right_left_tree;
            new_left_node.left_node = left_tree;
            self.left_node = new_left_tree;

            if let Some(node) = self.left_node.as_mut() {
                node.update_height();
            }

            self.update_height();

            true
        } else {
            false
        }
    }

    pub fn rotate_right(&mut self) -> bool {
        if let Some(left_node) = &mut self.left_node {
            let left_right_tree = left_node.right_node.take();
            let left_left_tree = left_node.left_node.take();

            let mut new_right_tree = mem::replace(&mut self.left_node, left_left_tree);
            mem::swap(&mut self.value, &mut new_right_tree.as_mut().unwrap().value);
            let right_tree = self.right_node.take();

            let new_right_node = new_right_tree.as_mut().unwrap();
            new_right_node.left_node = left_right_tree;
            new_right_node.right_node = right_tree;
            self.right_node = new_right_tree;

            if let Some(node) = self.right_node.as_mut() {
                node.update_height();
            }

            self.update_height();

            true
        } else {
            false
        }
    }

    pub fn rebalance(&mut self) -> bool {
        match self.balance_factor() {
            -2 => {
                if let Some(right_node) = &mut self.right_node {
                    if right_node.balance_factor() == 1 {
                        right_node.rotate_right();
                    }

                    self.rotate_left();

                    true
                } else {
                    unreachable!("Right node should exist if height == -2")
                }
            }

            2 => {
                if let Some(left_node) = &mut self.left_node {
                    if left_node.balance_factor() == -1 {
                        left_node.rotate_left();
                    }

                    self.rotate_right();

                    true
                } else {
                    unreachable!("Left node should exist if height == 2")
                }
            }
            _ => false,
        }
    }
}

pub(crate) trait NodeTrait {
    /// Return bytes representation of `Self`
    fn bytes(&self) -> Vec<u8>;

    /// Clone `Self` without reference to other nodes
    fn shallow_clone(&self) -> Self;
}

impl NodeTrait for Node {
    fn bytes(&self) -> Vec<u8> {
        let mut serialized = self.height.encode_var_vec();
        serialized.extend(self.size.encode_var_vec());
        serialized.extend(self.version.encode_var_vec());
        serialized.extend(encode_bytes(&self.key));
        serialized.extend(encode_bytes(&self.left_hash()));
        serialized.extend(encode_bytes(&self.right_hash()));

        serialized
    }

    fn shallow_clone(&self) -> Self {
        Self {
            left_node: None,
            right_node: None,
            key: self.key.clone(),
            height: self.height,
            size: self.size,
            hash: self.hash,
            version: self.version,
            value: self.value.clone(),
        }
    }
}

// TODO: rename loaded_version to head_version introduce a working_version (+ remove redundant loaded_version?). this will allow the first committed version to be version 0 rather than 1 (there is no version 0 currently!)
#[derive(Debug, Default)]
pub struct AvlTree {
    root: Option<Box<Node>>,
}

impl AvlTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: impl AsRef<[u8]>) -> Option<&NodeValue> {
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

    pub fn insert(&mut self, node: Node) -> bool {
        let mut prev_ptrs = Vec::<*mut Node>::new();
        let mut current_tree = &mut self.root;

        while let Some(current_node) = current_tree {
            prev_ptrs.push(&mut **current_node);

            match current_node.hash.cmp(&node.hash) {
                Ordering::Less => current_tree = &mut current_node.right_node,
                Ordering::Equal => {
                    return false;
                }
                Ordering::Greater => current_tree = &mut current_node.left_node,
            }
        }

        *current_tree = Some(Box::new(node));

        for node_ptr in prev_ptrs.into_iter().rev() {
            let node = unsafe { &mut *node_ptr };
            node.update_height();
            node.rebalance();
        }

        true
    }

    pub fn take(&mut self, hash: &[u8; HASH_LENGHT]) -> Option<NodeValue> {
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
                parent_left_node.rebalance();

                for node_ptr in inner_ptrs.into_iter().rev() {
                    let node = unsafe { &mut *node_ptr };
                    node.update_height();
                    node.rebalance();
                }

                target_node.update_height();
                target_node.rebalance();

                inner_value
            }
        };

        for node_ptr in prev_ptrs.into_iter().rev() {
            let node = unsafe { &mut *node_ptr };
            node.update_height();
            node.rebalance();
        }

        Some(taken_value)
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

fn encode_bytes(bz: &[u8]) -> Vec<u8> {
    let mut enc_bytes = bz.len().encode_var_vec();
    enc_bytes.extend_from_slice(bz);

    enc_bytes
}

fn decode_bytes(bz: &[u8]) -> Result<(Vec<u8>, usize), Error> {
    let (bz_length, n_consumed) = usize::decode_var(bz).ok_or(Error::NodeDeserialize)?;
    let bytes = bz[n_consumed..n_consumed + bz_length].to_vec();

    Ok((bytes, n_consumed + bz_length))
}

#[cfg(test)]
mod tests {
    //TODO
}
