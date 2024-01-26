use std::{cmp::Ordering, mem};

use super::node_v2::{NodeEnum, NodeTrait};

#[derive(Debug, Default)]
pub struct AvlTree {
    pub(crate) root: Option<Box<NodeEnum>>,
}

impl AvlTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_node(root: NodeEnum) -> Self {
        Self {
            root: Some(Box::new(root)),
        }
    }

    pub fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<&NodeEnum>, NodeError> {
        // Option<&Box<T>> -> Option<&T>
        let mut current_tree: Option<&NodeEnum> = match &self.root {
            Some(var) => Some(&*var),
            None => None,
        };

        while let Some(current_node) = current_tree {
            match current_node {
                NodeEnum::Inner(inner) => {
                    match inner.key().cmp(key.as_ref()) {
                        Ordering::Less => {
                            current_tree = current_node.right_node();
                        }
                        Ordering::Equal => {
                            return Ok(Some(&current_node));
                        }
                        Ordering::Greater => {
                            current_tree = current_node.left_node();
                        }
                    };
                }
                NodeEnum::Leaf(leaf) => {
                    return match leaf.key() == key.as_ref() {
                        true => Ok(Some(current_node)),
                        false => {
                            let mut err_key = Vec::new();

                            err_key.extend_from_slice(key.as_ref());

                            Err(NodeError::SearchAfter(format!("Error key: {:?}", err_key)))
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    pub fn set(
        &mut self,
        key: &impl AsRef<[u8]>,
        value: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, NodeError> {
        // Option<&mut Box<T>> -> Option<&mut T>
        let mut current_tree: Option<&mut NodeEnum> = match &mut self.root {
            Some(var) => Some(&mut *var),
            None => None,
        };

        while let Some(current_node) = current_tree {
            match current_node {
                NodeEnum::Inner(inner) => {
                    match inner.key().cmp(key.as_ref()) {
                        Ordering::Less => {
                            current_tree = current_node.right_node_mut();
                        }
                        Ordering::Equal => {
                            let mut err_key = Vec::new();
                            err_key.extend_from_slice(key.as_ref());

                            return Err(NodeError::ValueChange(format!(
                                "Error key: {:?}",
                                err_key
                            )));
                        }
                        Ordering::Greater => {
                            current_tree = current_node.left_node_mut();
                        }
                    };
                }
                NodeEnum::Leaf(leaf) => {
                    return match leaf.key() == key.as_ref() {
                        true => Ok(Some(mem::replace(&mut leaf.value, value))),
                        false => {
                            let mut err_key = Vec::new();
                            err_key.extend_from_slice(key.as_ref());

                            Err(NodeError::SearchAfter(format!("Error key: {:?}", err_key)))
                        }
                    };
                }
            }
        }

        Ok(None)
    }

    pub fn insert(&mut self, node: NodeEnum) -> bool {
        // TODO: I assume that we could make leaf node into inner for this

        let node = Box::new(node);
        return tree_insert(&mut self.root, node);

        fn tree_insert(tree: &mut Option<Box<NodeEnum>>, node: Box<NodeEnum>) -> bool {
            match tree {
                None => {
                    *tree = Some(node);
                    true
                }
                Some(tree_node) => {
                    let inserted = match tree_node.key().cmp(&node.key()) {
                        Ordering::Equal => false,
                        Ordering::Less => match tree_node.as_mut() {
                            NodeEnum::Inner(inner) => tree_insert(&mut inner.right_node, node),
                            NodeEnum::Leaf(_leaf) => {
                                tree_node.make_inner();
                                let inner = tree_node
                                    .inner_mut()
                                    .expect("Leaf node already parsed into inner");
                                tree_insert(&mut inner.right_node, node) //TODO: Is I should set it manually or call method?
                            }
                        },
                        Ordering::Greater => match tree_node.as_mut() {
                            NodeEnum::Inner(inner) => tree_insert(&mut inner.left_node, node),
                            NodeEnum::Leaf(_leaf) => {
                                tree_node.make_inner();
                                let inner = tree_node
                                    .inner_mut()
                                    .expect("Leaf node already parsed into inner");
                                tree_insert(&mut inner.left_node, node)
                            }
                        },
                    };
                    if inserted {
                        tree_node.update_height();
                        tree_node.update_size();
                        // tree_node.rebalance();
                    }
                    inserted
                }
            }
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum NodeError {
    #[error("Tried to change value of inner node. {0}")]
    ValueChange(String),
    #[error("Tried to search node after leaf node. {0}")]
    SearchAfter(String),
}
