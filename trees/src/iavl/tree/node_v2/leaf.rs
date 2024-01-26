use crate::merkle::EMPTY_HASH;

use super::{inner::InnerNode, Node, NodeEnum, NodeHash, NodeTrait};

#[derive(Debug, Clone)]
pub struct LeafNode {
    // Leaf nodes have size = 1 and height = 0
    pub(crate) node: Node,
    pub(crate) value: Vec<u8>,
}

impl From<InnerNode> for LeafNode {
    fn from(value: InnerNode) -> Self {
        Self {
            node: value.node,
            value: Vec::new(),
        }
    }
}

impl NodeTrait for LeafNode {
    fn hash(&self) -> &NodeHash {
        match &self.node.hash {
            Some(var) => var,
            None => &EMPTY_HASH,
        }
    }

    fn left_hash(&self) -> Option<&NodeHash> {
        None
    }

    fn right_hash(&self) -> Option<&NodeHash> {
        None
    }

    fn height(&self) -> u32 {
        0
    }

    fn left_height(&self) -> u32 {
        0
    }

    fn right_height(&self) -> u32 {
        0
    }

    fn size(&self) -> u8 {
        1
    }

    fn left_size(&self) -> u8 {
        0
    }

    fn right_size(&self) -> u8 {
        0
    }

    fn update_size(&mut self) {}

    fn update_height(&mut self) {}

    fn right_node_mut(&mut self) -> Option< &mut NodeEnum> {
        None
    }

    fn left_node_mut(&mut self) -> Option< &mut NodeEnum> {
        None
    }

    fn right_node_take(&mut self) -> Option<Box<NodeEnum>> {
        None
    }

    fn left_node_take(&mut self) -> Option<Box<NodeEnum>> {
        None
    }

    fn key(&self) ->  &[u8] {
        &self.node.key
    }

    fn right_node(&self) -> Option< &NodeEnum> {
        None
    }

    fn left_node(&self) -> Option< &NodeEnum> {
        None
    }
}
