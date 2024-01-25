pub mod inner;
pub mod leaf;

/*
    # HASHING
    A node's hash is calculated by hashing the height, size, and version of the node.
    If the node is a leaf node, then the key and value are also hashed.
    If the node is an inner node, the leftHash and rightHash are included in hash but the key is not.

    ## WHEN
    Hash is calculated when we call `SaveVersion` on tree
*/

use std::mem;

use enum_dispatch::enum_dispatch;

use crate::iavl::HASH_LENGHT;

use self::inner::{InnerNode, LeafNode};

pub type NodeHash = [u8; HASH_LENGHT];

#[derive(Debug, Clone)]
pub struct NodeKey {
    /// Version of the AVL Tree that this node was first added in
    first_version: u32,
    // local nonce for the same version
    // nonce : i32,
}

#[derive(Debug, Clone)]
pub struct Node {
    node_key: NodeKey,
    hash: Option<NodeHash>, // NOTE: New nodes have no hash.
    pub(crate) key: Vec<u8>,
}

#[enum_dispatch(NodeTrait)]
#[derive(Debug, Clone)]
pub enum NodeEnum {
    Inner(InnerNode),
    Leaf(LeafNode),
}

impl NodeEnum {
    pub fn inner_mut(&mut self) -> Option< &mut InnerNode >
    {
        match self {
            NodeEnum::Inner( var ) => Some( var ),
            NodeEnum::Leaf(_) => None,
        }
    }

    pub fn as_inner(self) -> InnerNode {
        match self {
            NodeEnum::Inner(var) => var,
            NodeEnum::Leaf(var) => var.into(),
        }
    }

    pub fn as_leaf(self) -> LeafNode {
        match self {
            NodeEnum::Inner(var) => var.into(),
            NodeEnum::Leaf(var) => var,
        }
    }

    pub fn rotate_left(&mut self) -> bool {
        match self {
            NodeEnum::Inner(inner) => {
                if let Some(right_node) = &mut inner.right_node {
                    let right_left_tree = right_node.left_node_take();
                    let right_right_tree = right_node.right_node_take();
 
                    let mut new_left_tree = mem::replace( &mut self.inner_mut().expect( "Node shoud be inner").right_node, right_right_tree);
                    // mem::swap(&mut self.value, &mut new_left_tree.as_mut().unwrap().value);
                    let left_tree = self.left_node_take();

                    let new_left_node = new_left_tree.as_mut().expect( "Node is Some").inner_mut().expect( "Node shoud be inner");
                    new_left_node.right_node = right_left_tree;
                    new_left_node.left_node = left_tree;

                    self.update_size();
                    self.update_height();

                    true
                } else {
                    false
                }
            }
            NodeEnum::Leaf(_leaf) => 
            { 
                // TODO: What should I do if node is leaf. Is this even possible?
                unreachable!( "Leaf node can't be rotated. Investigate state")
            }
        }
    }

    pub fn rotate_right(&mut self) -> bool {
        match self {
            NodeEnum::Inner(inner) => {
                if let Some(left_node) = &mut inner.left_node {
                    let left_left_tree = left_node.left_node_take();
                    let left_right_tree = left_node.right_node_take();
 
                    let mut new_right_tree = mem::replace( &mut self.inner_mut().expect( "Node shoud be inner").right_node, left_right_tree);
                    // mem::swap(&mut self.value, &mut new_left_tree.as_mut().unwrap().value);
                    let left_tree = self.right_node_take();

                    let new_right_node = new_right_tree.as_mut().expect( "Node is Some").inner_mut().expect( "Node shoud be inner");
                    new_right_node.right_node = left_left_tree;
                    new_right_node.left_node = left_tree;

                    self.update_size();
                    self.update_height();

                    true
                } else {
                    false
                }
            }
            NodeEnum::Leaf(_leaf) => 
            { 
                // TODO: What should I do if node is leaf. Is this even possible?
                unreachable!( "Leaf node can't be rotated. Investigate state")
            }
        }
    }
}

#[enum_dispatch]
pub trait NodeTrait {
    /// Return hash of node.
    /// [`crate::merkle::EMPTY_HASH`] if this is new node.
    fn hash(&self) -> &NodeHash;

    /// Return hash of left node.
    /// [`Option::None`] if left node is leaf
    fn left_hash(&self) -> Option<&NodeHash>;

    /// Return hash of right node.
    /// [`Option::None`] if right node is leaf
    fn right_hash(&self) -> Option<&NodeHash>;

    /// Cosmos: `subtreeHeight`.
    /// height of the node. [`LeafNode`] nodes have height 0
    fn height(&self) -> u32;

    fn left_height(&self) -> u32;

    fn right_height(&self) -> u32;

    fn size(&self) -> u8;

    fn left_size(&self) -> u8;

    fn right_size(&self) -> u8;

    fn update_size(&mut self);

    fn update_height(&mut self);

    fn balance_factor(&self) -> i8 {
        let left_height = self.left_height();
        let right_height = self.right_height();

        if left_height >= right_height {
            (left_height - right_height) as i8
        } else {
            -((right_height - left_height) as i8)
        }
    }

    fn right_node_take( &mut self ) -> Option< Box<NodeEnum>>;
    fn left_node_take( &mut self ) -> Option< Box< NodeEnum >>;

    fn right_node_mut( &mut self ) -> Option< &mut NodeEnum>;
    fn left_node_mut( &mut self ) -> Option< &mut NodeEnum>;
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("TODO: Clarify what could occur here")]
pub struct DeserializeError;
pub(crate) trait Writable {
    type Output;

    /// Deserialize node data from bytes
    fn deserialize(bytes: Vec<u8>) -> Result<Self::Output, DeserializeError>;

    /// Serialize node into bytes
    fn serialize(&self) -> Vec<u8>;
}
