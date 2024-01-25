/*
    # HASHING
    A node's hash is calculated by hashing the height, size, and version of the node.
    If the node is a leaf node, then the key and value are also hashed.
    If the node is an inner node, the leftHash and rightHash are included in hash but the key is not.

    ## WHEN
    Hash is calculated when we call `SaveVersion` on tree
*/

use enum_dispatch::enum_dispatch;

use crate::{iavl::HASH_LENGHT, merkle::EMPTY_HASH};

type NodeHash = [u8; HASH_LENGHT];

#[derive(Debug, Clone)]
pub struct NodeKey {
    /// Version of the AVL Tree that this node was first added in
    first_version: u32,
    // local nonce for the same version
    // nonce : i32,
}

#[derive(Debug, Clone)]
pub struct InnerNode {
    pub(crate) node: Node,
    pub(crate) left_node: Option<Box<NodeEnum>>,
    pub(crate) right_node: Option<Box<NodeEnum>>,
}

#[derive(Debug, Clone)]
pub struct LeafNode {
    pub(crate) node: Node,
    pub(crate) value: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Node {
    height: u8,
    size: u32, // NOTE: number of nodes in this subtree. If it exceeds 2 than tree should be rotated
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
