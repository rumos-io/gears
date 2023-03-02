use integer_encoding::VarInt;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub enum Node {
    Leaf(LeafNode),
    Inner(InnerNode),
}

#[derive(Debug, Clone)]
pub struct InnerNode {
    pub left_node: Box<Node>,
    pub right_node: Box<Node>,
    pub key: Vec<u8>,
    pub height: u8,
    pub size: u32, // number of leaf nodes in this node's subtree
    pub left_hash: [u8; 32],
    pub right_hash: [u8; 32],
    pub version: u32,
}

#[derive(Debug, Clone)]
pub struct LeafNode {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub version: u32,
}

impl Node {
    pub fn get_key(&self) -> &Vec<u8> {
        match self {
            Node::Leaf(n) => &n.key,
            Node::Inner(n) => &n.key,
        }
    }
    pub fn hash(&self) -> [u8; 32] {
        let serialized = self.serialize();
        Sha256::digest(serialized).into()
    }
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Node::Leaf(node) => {
                let height: i64 = 0; // i64 required for compatibility with cosmos
                let mut node_bytes = height.encode_var_vec();

                let size: i64 = 1; // i64 required for compatibility with cosmos
                node_bytes.append(&mut size.encode_var_vec());

                let version: i64 = node.version.into(); // conversion to i64 required for compatibility with cosmos
                node_bytes.append(&mut version.encode_var_vec());
                node_bytes.append(&mut encode_bytes(node.key.to_vec()));

                // Indirection is needed to provide proofs without values.
                let mut hasher = Sha256::new();
                hasher.update(node.value.clone());
                let hashed_value = hasher.finalize();

                node_bytes.append(&mut encode_bytes(hashed_value.to_vec()));

                return node_bytes;
            }
            Node::Inner(node) => {
                let height: i64 = node.height.into(); // conversion to i64 required for compatibility with cosmos
                let mut node_bytes = height.encode_var_vec();

                let size: i64 = node.size.into(); // conversion to i64 required for compatibility with cosmos
                node_bytes.append(&mut size.encode_var_vec());

                let version: i64 = node.version.into(); // conversion to i64 required for compatibility with cosmos
                node_bytes.append(&mut version.encode_var_vec());

                node_bytes.append(&mut encode_bytes(node.left_hash.clone().into()));

                node_bytes.append(&mut encode_bytes(node.right_hash.clone().into()));

                return node_bytes;
            }
        }
    }
}

fn encode_bytes(mut bz: Vec<u8>) -> Vec<u8> {
    let mut enc_bytes = bz.len().encode_var_vec();

    enc_bytes.append(&mut bz);

    return enc_bytes;
}
