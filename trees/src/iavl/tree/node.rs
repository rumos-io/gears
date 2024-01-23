use std::{cmp, mem};

use integer_encoding::VarInt;

use crate::{iavl::HASH_LENGHT, merkle::EMPTY_HASH, Error};

#[derive(Debug, Clone)]
pub struct Node {
    height: u8,
    size: u32, // number of nodes in this subtree. If it exceeds 2 than tree should be rotated
    version: u32,
    pub(crate) key: Vec<u8>,
    pub(crate) value: Vec<u8>, //TODO: It needs to be made Option to say that value located in DB. Or I will get better idea
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

    /// TODO
    fn hash(&self) -> &[u8; HASH_LENGHT];

    type Output;

    /// TODO
    fn deserialize(bytes: Vec<u8>) -> Result<Self::Output, Error>;

    /// TODO
    fn serialize(&self) -> Vec<u8>;
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

    type Output = Self;

    fn deserialize(bytes: Vec<u8>) -> Result<Self::Output, Error> {
        let (height, mut n) = u8::decode_var(&bytes).ok_or(Error::NodeDeserialize)?;
        let (size, ns) = u32::decode_var(&bytes[n..]).ok_or(Error::NodeDeserialize)?;
        n += ns;
        let (version, nv) = u32::decode_var(&bytes[n..]).ok_or(Error::NodeDeserialize)?;
        n += nv;
        let (key, nk) = decode_bytes(&bytes[n..])?;
        n += nk;
        let (value, nl) = decode_bytes(&bytes[n..])?;
        n += nl;
        let (ser_hash, _) = decode_bytes(&bytes[n..])?;

        let mut hash: [u8; HASH_LENGHT] = Default::default();
        if hash.len() != ser_hash.len() {
            Err(Error::NodeDeserialize)?
        }
        hash.copy_from_slice(&ser_hash);

        Ok(Node {
            left_node: None,
            right_node: None,
            key,
            height,
            size,
            version,
            value,
            hash,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut serialized = self.height.encode_var_vec();
        serialized.extend(self.size.encode_var_vec());
        serialized.extend(self.version.encode_var_vec());
        serialized.extend(encode_bytes(&self.key));
        serialized.extend(encode_bytes(&self.value));
        serialized.extend(encode_bytes(&self.hash));

        serialized
    }

    fn hash(&self) -> &[u8; HASH_LENGHT] {
        &self.hash
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
