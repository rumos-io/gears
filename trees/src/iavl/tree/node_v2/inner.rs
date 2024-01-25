use crate::merkle::EMPTY_HASH;

pub use super::leaf::LeafNode;
pub use super::{Node, NodeEnum, NodeHash, NodeTrait};

#[derive(Debug, Clone)]
pub struct InnerNode {
    size: u8, // NOTE: number of nodes in this subtree. If it exceeds 2 than tree should be rotated.
    height: u32, // subtreeHeight. height of the node.
    pub(crate) node: Node,
    pub(crate) left_node: Option<Box<NodeEnum>>,
    pub(crate) right_node: Option<Box<NodeEnum>>,
}

impl From<LeafNode> for InnerNode {
    fn from(value: LeafNode) -> Self {
        Self {
            size: 1,
            height: 0,
            node: value.node,
            left_node: None,
            right_node: None,
        }
    }
}

impl NodeTrait for InnerNode {
    fn hash(&self) -> &NodeHash {
        match &self.node.hash {
            Some(var) => var,
            None => &EMPTY_HASH,
        }
    }

    fn left_hash(&self) -> Option<&NodeHash> {
        self.left_node
            .as_ref()
            .map_or(Some(&EMPTY_HASH), |this| this.left_hash())
    }

    fn right_hash(&self) -> Option<&NodeHash> {
        self.right_node
            .as_ref()
            .map_or(Some(&EMPTY_HASH), |this| this.left_hash())
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn left_height(&self) -> u32 {
        self.left_node.as_ref().map_or(0, |this| this.height())
    }

    fn right_height(&self) -> u32 {
        self.right_node.as_ref().map_or(0, |this| this.height())
    }

    fn size(&self) -> u8 {
        self.size
    }

    fn left_size(&self) -> u8 {
        self.left_node.as_ref().map_or(0, |this| this.size())
    }

    fn right_size(&self) -> u8 {
        self.right_node.as_ref().map_or(0, |this| this.size())
    }

    fn update_size(&mut self) {
        self.size = self.left_size() + self.right_size()
    }

    fn update_height(&mut self) {
        self.height = 1 + std::cmp::max(self.left_height(), self.right_height())
    }

    fn right_node_take(&mut self) -> Option<Box<NodeEnum>> {
        self.right_node.take()
    }

    fn left_node_take(&mut self) -> Option<Box<NodeEnum>> {
        self.left_node.take()
    }

    fn right_node_mut(&mut self) -> Option< &mut NodeEnum> {
        match &mut self.right_node 
        {
            Some( var ) => Some( var ),
            None => None,
        }
    }

    fn left_node_mut(&mut self) -> Option< &mut NodeEnum> {
        match &mut self.left_node 
        {
            Some( var ) => Some( var ),
            None => None,
        }
    }


}
