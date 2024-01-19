use std::{
    cmp,
    collections::BTreeSet,
    mem,
    ops::{Bound, RangeBounds},
};

use database::Database;
use enum_dispatch::enum_dispatch;
use integer_encoding::VarInt;
use sha2::{Digest, Sha256};

use crate::{error::Error, merkle::EMPTY_HASH};

use super::node_db::NodeDB;

#[enum_dispatch(NodeValue)]
pub trait NodeValueTrait {
    fn bytes(&self) -> Vec<u8>;
}

// TODO: More general variant
impl NodeValueTrait for Vec<u8> {
    fn bytes(&self) -> Vec<u8> {
        self.clone()
    }
}

/// Value which stored in AVL Tree.
#[enum_dispatch]
#[derive(Debug, Clone, PartialEq)] // TODO: what we could remove from here?
pub enum NodeValue {
    Bytes(Vec<u8>),
    // TODO: Ask Kevin about other values
}

impl Default for NodeValue {
    fn default() -> Self {
        Self::Bytes(Vec::new())
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DbValue;

impl Drop for DbValue {
    fn drop(&mut self) {
        // Probably this where we should delete value from db
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct Node {
    pub(crate) left_node: Option<Box<Node>>, // None means value is the same as what's in the DB
    pub(crate) right_node: Option<Box<Node>>,
    pub(crate) value: NodeValue,
    pub(crate) key: Vec<u8>,
    height: u8,
    size: u32, // number of nodes in this subtree. If it exceeds 2 than tree should be rotated
    pub(crate) left_hash: [u8; 32],
    pub(crate) right_hash: [u8; 32],
    version: u32,
}

impl Node {
    pub fn hash(&self) -> [u8; 32] {
        let serialized = self.hash_serialize();
        Sha256::digest(serialized).into()
    }

    fn hash_serialize(&self) -> Vec<u8> {
        // NOTE: i64 is used here for parameters for compatibility wih cosmos
        let height: i64 = self.height.into();
        let size: i64 = self.size.into();
        let version: i64 = self.version.into();

        let mut serialized = height.encode_var_vec();
        serialized.extend(size.encode_var_vec());
        serialized.extend(version.encode_var_vec());
        serialized.extend(encode_bytes(&self.left_hash));
        serialized.extend(encode_bytes(&self.right_hash));

        serialized
    }

    pub(crate) fn serialize(&self) -> Vec<u8> {
        let mut serialized = self.height.encode_var_vec();
        serialized.extend(self.size.encode_var_vec());
        serialized.extend(self.version.encode_var_vec());
        serialized.extend(encode_bytes(&self.key));
        serialized.extend(encode_bytes(&self.left_hash));
        serialized.extend(encode_bytes(&self.right_hash));

        serialized
    }

    pub(crate) fn deserialize(bytes: Vec<u8>) -> Result<Self, Error> {
        let (height, mut n) = u8::decode_var(&bytes).ok_or(Error::NodeDeserialize)?;
        let (size, ns) = u32::decode_var(&bytes[n..]).ok_or(Error::NodeDeserialize)?;
        n += ns;
        let (version, nv) = u32::decode_var(&bytes[n..]).ok_or(Error::NodeDeserialize)?;
        n += nv;
        let (key, nk) = decode_bytes(&bytes[n..])?;
        n += nk;

        let (left_hash, nl) = decode_bytes(&bytes[n..])?;
        n += nl;
        let (right_hash, _) = decode_bytes(&bytes[n..])?;
        Ok(Node {
            left_node: None,
            right_node: None,
            key,
            height,
            size,
            left_hash: left_hash.try_into().map_err(|_| Error::NodeDeserialize)?,
            right_hash: right_hash.try_into().map_err(|_| Error::NodeDeserialize)?,
            version,
            value: Default::default(), // TODO: Probably what earlier was right_hash now should be this
        })
    }

    fn get_mut_left_node<T: Database>(&mut self, node_db: &NodeDB<T>) -> &mut Node {
        self.left_node.get_or_insert_with(|| {
            let node = node_db
                .get_node(&self.left_hash)
                .expect("node should be in db");
            Box::new(node)
        })
    }

    fn get_mut_right_node<T: Database>(&mut self, node_db: &NodeDB<T>) -> &mut Node {
        self.right_node.get_or_insert_with(|| {
            let node = node_db
                .get_node(&self.right_hash)
                .expect("node should be in db");

            Box::new(node)
        })
    }

    fn update_left_hash(&mut self) {
        if let Some(left_node) = &self.left_node {
            self.left_hash = left_node.hash();
        }
    }

    fn update_right_hash(&mut self) {
        if let Some(node) = &self.right_node {
            self.right_hash = node.hash();
        }
    }

    /// This does three things at once to prevent repeating the same process for getting the left and right nodes
    fn update_height_and_size_get_balance_factor<T: Database>(
        &mut self,
        node_db: &NodeDB<T>,
    ) -> i16 {
        let (left_height, left_size) = match &self.left_node {
            Some(left_node) => (left_node.height, left_node.size),
            None => {
                let left_node = node_db
                    .get_node(&self.left_hash)
                    .expect("node db should contain all nodes");

                (left_node.height, left_node.size)
            }
        };

        let (right_height, right_size) = match &self.right_node {
            Some(right_node) => (right_node.height, right_node.size),
            None => {
                let right_node = node_db
                    .get_node(&self.right_hash)
                    .expect("node db should contain all nodes");

                (right_node.height, right_node.size)
            }
        };

        self.height = 1 + cmp::max(left_height, right_height);
        self.size = left_size + right_size;

        left_height as i16 - right_height as i16
    }

    // TODO: What is 'shallow' here?
    pub(crate) fn shallow_clone(&self) -> Self {
        Self {
            left_node: None,
            right_node: None,
            key: self.key.clone(),
            height: self.height,
            size: self.size,
            left_hash: self.left_hash,
            right_hash: self.right_hash,
            version: self.version,
            value: self.value.clone(),
        }
    }
}

// #[derive(Debug, Clone, PartialEq, Default)]
// pub(crate) struct LeafNode {
//     pub(crate) key: Vec<u8>,
//     pub(crate) value: Vec<u8>,
//     version: u32,
// }

// #[derive(Debug, Clone, PartialEq)]
// pub(crate) enum Node {
//     Leaf(LeafNode),
//     Inner(InnerNode),
// }

// impl Default for Node {
//     fn default() -> Self {
//         Node::Leaf(Default::default())
//     }
// }

// impl Node {
//     pub(crate) fn shallow_clone(&self) -> Node {
//         match self {
//             Node::Leaf(n) => Node::Leaf(n.clone()),
//             Node::Inner(n) => Node::Inner(n.shallow_clone()),
//         }
//     }
//     pub fn get_key(&self) -> &Vec<u8> {
//         match self {
//             Node::Leaf(leaf) => &leaf.key,
//             Node::Inner(inner) => &inner.key,
//         }
//     }

//     pub fn get_height(&self) -> u8 {
//         match self {
//             Node::Leaf(_) => 0,
//             Node::Inner(inner) => inner.height,
//         }
//     }

//     pub fn new_leaf(key: Vec<u8>, value: Vec<u8>, version: u32) -> Node {
//         Node::Leaf(LeafNode {
//             key,
//             value,
//             version,
//         })
//     }

//     pub fn hash(&self) -> [u8; 32] {
//         let serialized = self.hash_serialize();
//         Sha256::digest(serialized).into()
//     }

//     fn hash_serialize(&self) -> Vec<u8> {
//         match &self {
//             Node::Leaf(node) => {
//                 // NOTE: i64 is used here for parameters for compatibility wih cosmos
//                 let height: i64 = 0;
//                 let size: i64 = 1;
//                 let version: i64 = node.version.into();
//                 let hashed_value = Sha256::digest(&node.value);

//                 let mut serialized = height.encode_var_vec();
//                 serialized.extend(size.encode_var_vec());
//                 serialized.extend(version.encode_var_vec());
//                 serialized.extend(encode_bytes(&node.key));
//                 serialized.extend(encode_bytes(&hashed_value));

//                 serialized
//             }
//             Node::Inner(node) => {
//                 // NOTE: i64 is used here for parameters for compatibility wih cosmos
//                 let height: i64 = node.height.into();
//                 let size: i64 = node.size.into();
//                 let version: i64 = node.version.into();

//                 let mut serialized = height.encode_var_vec();
//                 serialized.extend(size.encode_var_vec());
//                 serialized.extend(version.encode_var_vec());
//                 serialized.extend(encode_bytes(&node.left_hash));
//                 serialized.extend(encode_bytes(&node.right_hash));

//                 serialized
//             }
//         }
//     }

//     pub(crate) fn serialize(&self) -> Vec<u8> {
//         match &self {
//             Node::Leaf(node) => {
//                 let height: u8 = 0;
//                 let size: u32 = 1;

//                 let mut serialized = height.encode_var_vec();
//                 serialized.extend(size.encode_var_vec());
//                 serialized.extend(node.version.encode_var_vec());
//                 serialized.extend(encode_bytes(&node.key));
//                 serialized.extend(encode_bytes(&node.value));

//                 serialized
//             }
//             Node::Inner(node) => {
//                 let mut serialized = node.height.encode_var_vec();
//                 serialized.extend(node.size.encode_var_vec());
//                 serialized.extend(node.version.encode_var_vec());
//                 serialized.extend(encode_bytes(&node.key));
//                 serialized.extend(encode_bytes(&node.left_hash));
//                 serialized.extend(encode_bytes(&node.right_hash));

//                 serialized
//             }
//         }
//     }

//     pub(crate) fn deserialize(bytes: Vec<u8>) -> Result<Self, Error> {
//         let (height, mut n) = u8::decode_var(&bytes).ok_or(Error::NodeDeserialize)?;
//         let (size, ns) = u32::decode_var(&bytes[n..]).ok_or(Error::NodeDeserialize)?;
//         n += ns;
//         let (version, nv) = u32::decode_var(&bytes[n..]).ok_or(Error::NodeDeserialize)?;
//         n += nv;
//         let (key, nk) = decode_bytes(&bytes[n..])?;
//         n += nk;

//         if height == 0 {
//             // leaf node
//             let (value, _) = decode_bytes(&bytes[n..])?;

//             Ok(Node::Leaf(LeafNode {
//                 key,
//                 value,
//                 version,
//             }))
//         } else {
//             // inner node
//             let (left_hash, nl) = decode_bytes(&bytes[n..])?;
//             n += nl;
//             let (right_hash, _) = decode_bytes(&bytes[n..])?;
//             Ok(Node::Inner(InnerNode {
//                 left_node: None,
//                 right_node: None,
//                 key,
//                 height,
//                 size,
//                 left_hash: left_hash.try_into().map_err(|_| Error::NodeDeserialize)?,
//                 right_hash: right_hash.try_into().map_err(|_| Error::NodeDeserialize)?,
//                 version,
//             }))
//         }
//     }

//     fn get_size(&self) -> u32 {
//         match &self {
//             Node::Leaf(_) => 1,
//             Node::Inner(n) => n.size,
//         }
//     }
// }

// TODO: rename loaded_version to head_version introduce a working_version (+ remove redundant loaded_version?). this will allow the first committed version to be version 0 rather than 1 (there is no version 0 currently!)
#[derive(Debug)]
pub struct Tree<T> {
    root: Option<Node>,
    pub(crate) node_db: NodeDB<T>,
    pub(crate) loaded_version: u32,
    pub(crate) versions: BTreeSet<u32>,
}

impl<T> Tree<T>
where
    T: Database,
{
    /// Panics if cache_size=0
    pub fn new(db: T, target_version: Option<u32>, cache_size: usize) -> Result<Tree<T>, Error> {
        assert!(cache_size > 0);
        let node_db = NodeDB::new(db, cache_size);
        let versions = node_db.get_versions();

        if let Some(target_version) = target_version {
            let root = node_db.get_root_node(target_version)?;

            Ok(Tree {
                root,
                loaded_version: target_version,
                node_db,
                versions,
            })
        } else {
            // use the latest version available
            if let Some(latest_version) = versions.last() {
                Ok(Tree {
                    root: node_db
                        .get_root_node(*latest_version)
                        .expect("invalid data in database - possible database corruption"),
                    loaded_version: *latest_version,
                    node_db,
                    versions,
                })
            } else {
                Ok(Tree {
                    root: None,
                    loaded_version: 0,
                    node_db,
                    versions,
                })
            }
        }
    }

    /// Save the current tree to disk.
    /// Returns an error if saving would overwrite an existing version
    pub fn save_version(&mut self) -> Result<([u8; 32], u32), Error> {
        let version = self.loaded_version + 1;

        if self.versions.contains(&version) {
            // If the version already exists, return an error as we're attempting to overwrite.
            // However, the same hash means idempotent (i.e. no-op).
            // TODO: do we really need to be doing this?
            let saved_hash = self
                .node_db
                .get_root_hash(version)
                .expect("invalid data in database - possible database corruption");
            let working_hash = self.root_hash();

            if saved_hash == working_hash {
                self.loaded_version = version;

                // clear the root node's left and right nodes if they exist
                if let Some(node) = &mut self.root {
                    {
                        node.left_node = None;
                        node.right_node = None;
                    }
                }
                return Ok((saved_hash, self.loaded_version));
            }
            return Err(Error::Overwrite);
        }

        let root = self.root.as_mut();
        let root_hash = if let Some(root) = root {
            let root_hash = self.node_db.save_tree(root);
            self.node_db.save_version(version, &root_hash);
            root_hash
        } else {
            self.node_db.save_version(version, &EMPTY_HASH);
            EMPTY_HASH
        };

        self.versions.insert(version);

        self.loaded_version = version;
        Ok((root_hash, self.loaded_version))
    }

    pub fn root_hash(&self) -> [u8; 32] {
        match &self.root {
            Some(root) => root.hash(),
            None => EMPTY_HASH,
        }
    }

    pub fn loaded_version(&self) -> u32 {
        self.loaded_version
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        match &self.root {
            Some(root) => self.get_(key, root),
            None => None,
        }
    }

    fn get_(&self, key: &[u8], root: &Node) -> Option<Vec<u8>> {
        let mut loop_node = root;
        let mut cached_node;

        loop {
            if key < &root.key {
                match &root.left_node {
                    Some(left_node) => loop_node = left_node,
                    None => {
                        let left_node = self
                            .node_db
                            .get_node(&root.left_hash)
                            .expect("node db should contain all nodes");

                        cached_node = left_node;
                        loop_node = &cached_node;
                    }
                }
            } else {
                match &root.right_node {
                    Some(right_node) => loop_node = right_node,
                    None => {
                        let right_node = self
                            .node_db
                            .get_node(&root.right_hash)
                            .expect("node db should contain all nodes");

                        cached_node = right_node;
                        loop_node = &cached_node;
                    }
                }
            }
        }
    }

    // pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
    //     match &mut self.root {
    //         Some(root) => {
    //             Self::recursive_set(root, key, value, self.loaded_version + 1, &mut self.node_db)
    //         }
    //         None => {
    //             self.root = Some(Node::Leaf(LeafNode {
    //                 key,
    //                 value,
    //                 version: self.loaded_version + 1,
    //             }));
    //         }
    //     };
    // }

    // fn recursive_set(
    //     mut node: &mut Node,
    //     key: Vec<u8>,
    //     value: Vec<u8>,
    //     version: u32,
    //     node_db: &mut NodeDB<T>,
    // ) {
    //     match &mut node {
    //         Node::Leaf(leaf_node) => match key.cmp(&leaf_node.key) {
    //             cmp::Ordering::Less => {
    //                 let left_node = Node::new_leaf(key, value, version);
    //                 let left_hash = left_node.hash();
    //                 let right_node = Node::Leaf(leaf_node.clone());
    //                 let right_hash = right_node.hash();

    //                 *node = Node::Inner(InnerNode {
    //                     key: leaf_node.key.clone(),
    //                     left_node: Some(Box::new(left_node)),
    //                     right_node: Some(Box::new(right_node)),
    //                     height: 1,
    //                     size: 2,
    //                     version,
    //                     left_hash,
    //                     right_hash,
    //                 });
    //             }
    //             cmp::Ordering::Equal => {
    //                 leaf_node.value = value;
    //                 leaf_node.version = version;
    //             }
    //             cmp::Ordering::Greater => {
    //                 let right_node = Node::new_leaf(key.clone(), value, version);
    //                 let right_hash = right_node.hash();
    //                 let left_subtree = node.clone();
    //                 let left_hash = left_subtree.hash();

    //                 *node = Node::Inner(InnerNode {
    //                     key,
    //                     left_node: Some(Box::new(left_subtree)),
    //                     right_node: Some(Box::new(right_node)),
    //                     height: 1,
    //                     size: 2,
    //                     left_hash,
    //                     right_hash,
    //                     version,
    //                 });
    //             }
    //         },
    //         Node::Inner(root_node) => {
    //             // Perform normal BST
    //             if key < root_node.key {
    //                 Self::recursive_set(
    //                     root_node.get_mut_left_node(node_db),
    //                     key.clone(),
    //                     value,
    //                     version,
    //                     node_db,
    //                 );
    //                 root_node.update_left_hash();
    //             } else {
    //                 Self::recursive_set(
    //                     root_node.get_mut_right_node(node_db),
    //                     key.clone(),
    //                     value,
    //                     version,
    //                     node_db,
    //                 );
    //                 root_node.update_right_hash();
    //             }

    //             // Update height + size + version
    //             let balance_factor = root_node.update_height_and_size_get_balance_factor(node_db);
    //             root_node.version = version;

    //             // If the tree is unbalanced then try out the usual four cases
    //             if balance_factor > 1 {
    //                 let left_node = root_node.get_mut_left_node(node_db);

    //                 if &key < left_node.get_key() {
    //                     // Case 1 - Right
    //                     Self::right_rotate(node, version, node_db)
    //                         .expect("Given the imbalance, expect rotation to always succeed");
    //                 } else {
    //                     // Case 2 - Left Right
    //                     Self::left_rotate(left_node, version, node_db)
    //                         .expect("Given the imbalance, expect rotation to always succeed");
    //                     Self::right_rotate(node, version, node_db)
    //                         .expect("Given the imbalance, expect rotation to always succeed");
    //                 }
    //             } else if balance_factor < -1 {
    //                 let right_node = root_node.get_mut_right_node(node_db);

    //                 if &key > right_node.get_key() {
    //                     // Case 3 - Left
    //                     Self::left_rotate(node, version, node_db)
    //                         .expect("Given the imbalance, expect rotation to always succeed");
    //                 } else {
    //                     // Case 4 - Right Left
    //                     Self::right_rotate(right_node, version, node_db)
    //                         .expect("Given the imbalance, expect rotation to always succeed");
    //                     Self::left_rotate(node, version, node_db)
    //                         .expect("Given the imbalance, expect rotation to always succeed");
    //                 }
    //             }
    //         }
    //     }
    // }

    fn right_rotate(node: &mut Node, version: u32, node_db: &NodeDB<T>) -> Result<(), Error> {
        let mut z = mem::take(node);
        let mut y = mem::take(node.get_mut_left_node(node_db));

        let t3 = y.right_node;

        // Perform rotation on z and update height and hash
        z.left_node = t3;
        z.left_hash = y.right_hash;
        z.update_height_and_size_get_balance_factor(node_db);
        z.version = version;

        // Perform rotation on y, update hash and update height
        y.right_hash = z.hash();
        y.right_node = Some(Box::new(z));
        y.update_height_and_size_get_balance_factor(node_db);
        y.version = version;

        *node = y;

        Ok(())
    }

    fn left_rotate(node: &mut Node, version: u32, node_db: &NodeDB<T>) -> Result<(), Error> {
        let mut z = mem::take(node);
        let mut y = mem::take(z.get_mut_right_node(node_db));

        let t2 = y.left_node;

        // Perform rotation on z and update height and hash
        z.right_node = t2;
        z.right_hash = y.left_hash;
        z.update_height_and_size_get_balance_factor(node_db);
        z.version = version;

        // Perform rotation on y, update hash and update height
        y.left_hash = z.hash();
        y.left_node = Some(Box::new(z));
        y.update_height_and_size_get_balance_factor(node_db);
        y.version = version;

        *node = y;

        Ok(())
    }

    pub fn range<R>(&self, range: R) -> Range<'_, R, T>
    where
        R: RangeBounds<Vec<u8>>,
    {
        match &self.root {
            Some(root) => Range {
                range,
                delayed_nodes: vec![root.clone()], //TODO: remove clone
                node_db: &self.node_db,
            },
            None => Range {
                range,
                delayed_nodes: vec![],
                node_db: &self.node_db,
            },
        }
    }
}

pub struct Range<'a, R: RangeBounds<Vec<u8>>, T>
where
    T: Database,
{
    pub(crate) range: R,
    pub(crate) delayed_nodes: Vec<Node>,
    pub(crate) node_db: &'a NodeDB<T>,
}

impl<'a, T: RangeBounds<Vec<u8>>, R: Database> Range<'a, T, R> {
    fn traverse(&mut self) -> Option<(Vec<u8>, Vec<u8>)> {
        let node = self.delayed_nodes.pop()?;

        let after_start = match self.range.start_bound() {
            Bound::Included(l) => node.key > *l,
            Bound::Excluded(l) => node.key > *l,
            Bound::Unbounded => true,
        };

        let before_end = match self.range.end_bound() {
            Bound::Included(u) => node.key <= *u,
            Bound::Excluded(u) => node.key < *u,
            Bound::Unbounded => true,
        };

        // Traverse through the left subtree, then the right subtree.
        if before_end {
            match node.right_node {
                Some(right_node) => self.delayed_nodes.push(*right_node), //TODO: deref will cause a clone, remove
                None => {
                    let right_node = self
                        .node_db
                        .get_node(&node.right_hash)
                        .expect("node db should contain all nodes");

                    self.delayed_nodes.push(right_node);
                }
            }
        }

        if after_start {
            match node.left_node {
                Some(left_node) => self.delayed_nodes.push(*left_node), //TODO: deref will cause a clone, remove
                None => {
                    let left_node = self
                        .node_db
                        .get_node(&node.left_hash)
                        .expect("node db should contain all nodes");

                    //self.cached_nodes.push(left_node);
                    self.delayed_nodes.push(left_node);
                }
            }
        }

        self.traverse()
    }
}

impl<'a, T: RangeBounds<Vec<u8>>, R: Database> Iterator for Range<'a, T, R> {
    type Item = (Vec<u8>, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        self.traverse()
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
    // use std::borrow::Borrow;

    // use super::*;
    // use database::MemDB;

    // #[test]
    // fn right_rotate_works() {
    //     let t3 = InnerNode {
    //         left_node: Some(Box::new(Node::Leaf(LeafNode {
    //             key: vec![19],
    //             value: vec![3, 2, 1],
    //             version: 0,
    //         }))),
    //         right_node: Some(Box::new(Node::Leaf(LeafNode {
    //             key: vec![20],
    //             value: vec![1, 6, 9],
    //             version: 0,
    //         }))),
    //         key: vec![20],
    //         height: 1,
    //         size: 2,
    //         left_hash: [
    //             56, 18, 97, 18, 6, 216, 38, 113, 24, 103, 129, 119, 92, 30, 188, 114, 183, 100,
    //             110, 73, 39, 131, 243, 199, 251, 72, 125, 220, 56, 132, 125, 106,
    //         ],
    //         right_hash: [
    //             150, 105, 234, 135, 99, 29, 12, 162, 67, 236, 81, 117, 3, 18, 217, 76, 202, 161,
    //             168, 94, 102, 108, 58, 135, 122, 167, 228, 134, 150, 121, 201, 234,
    //         ],
    //         version: 0,
    //     };

    //     let y = InnerNode {
    //         left_node: Some(Box::new(Node::Leaf(LeafNode {
    //             key: vec![18],
    //             value: vec![3, 2, 1],
    //             version: 0,
    //         }))),
    //         right_node: Some(Box::new(Node::Inner(t3))),
    //         key: vec![19],
    //         height: 2,
    //         size: 3,
    //         left_hash: [
    //             93, 129, 120, 78, 65, 12, 13, 69, 115, 187, 137, 249, 49, 28, 235, 190, 117, 117,
    //             64, 156, 133, 127, 116, 73, 127, 31, 220, 155, 141, 243, 58, 254,
    //         ],
    //         right_hash: [
    //             192, 103, 168, 209, 21, 23, 137, 121, 173, 138, 179, 199, 124, 163, 200, 22, 101,
    //             85, 103, 102, 253, 118, 15, 195, 248, 223, 181, 228, 63, 234, 156, 135,
    //         ],
    //         version: 0,
    //     };

    //     let z = InnerNode {
    //         left_node: Some(Box::new(Node::Inner(y))),
    //         right_node: Some(Box::new(Node::Leaf(LeafNode {
    //             key: vec![21],
    //             value: vec![3, 2, 1],
    //             version: 0,
    //         }))),
    //         key: vec![21],
    //         height: 3,
    //         size: 4,
    //         left_hash: [
    //             99, 11, 87, 15, 142, 124, 184, 114, 169, 142, 60, 89, 127, 225, 44, 148, 55, 15,
    //             134, 99, 95, 20, 72, 212, 28, 163, 207, 203, 187, 144, 112, 183,
    //         ],
    //         right_hash: [
    //             0, 85, 79, 1, 62, 128, 35, 121, 122, 250, 9, 14, 106, 197, 49, 81, 58, 121, 9, 157,
    //             156, 44, 10, 204, 48, 235, 172, 20, 43, 158, 240, 254,
    //         ],
    //         version: 0,
    //     };

    //     let mut z = Node::Inner(z);

    //     let db = MemDB::new();
    //     Tree::right_rotate(&mut z, 0, &NodeDB::new(db, 100)).unwrap();

    //     let hash = z.hash();
    //     let expected = [
    //         69, 219, 80, 128, 205, 82, 236, 60, 148, 147, 20, 32, 93, 192, 39, 130, 142, 68, 139,
    //         82, 137, 143, 154, 101, 208, 126, 98, 136, 17, 60, 138, 232,
    //     ];
    //     assert_eq!(hash, expected)
    // }

    // #[test]
    // fn left_rotate_works() {
    //     let t2 = InnerNode {
    //         left_node: Some(Box::new(Node::Leaf(LeafNode {
    //             key: vec![19],
    //             value: vec![3, 2, 1],
    //             version: 0,
    //         }))),
    //         right_node: Some(Box::new(Node::Leaf(LeafNode {
    //             key: vec![20],
    //             value: vec![1, 6, 9],
    //             version: 0,
    //         }))),
    //         key: vec![20],
    //         height: 1,
    //         size: 2,
    //         left_hash: [
    //             56, 18, 97, 18, 6, 216, 38, 113, 24, 103, 129, 119, 92, 30, 188, 114, 183, 100,
    //             110, 73, 39, 131, 243, 199, 251, 72, 125, 220, 56, 132, 125, 106,
    //         ],
    //         right_hash: [
    //             150, 105, 234, 135, 99, 29, 12, 162, 67, 236, 81, 117, 3, 18, 217, 76, 202, 161,
    //             168, 94, 102, 108, 58, 135, 122, 167, 228, 134, 150, 121, 201, 234,
    //         ],
    //         version: 0,
    //     };

    //     let y = InnerNode {
    //         right_node: Some(Box::new(Node::Leaf(LeafNode {
    //             key: vec![21],
    //             value: vec![3, 2, 1, 1],
    //             version: 0,
    //         }))),
    //         left_node: Some(Box::new(Node::Inner(t2))),
    //         key: vec![21],
    //         height: 2,
    //         size: 3,
    //         right_hash: [
    //             228, 95, 46, 250, 156, 226, 109, 111, 149, 171, 184, 71, 170, 219, 77, 170, 113,
    //             216, 178, 65, 111, 142, 17, 195, 169, 129, 164, 6, 25, 91, 141, 173,
    //         ],
    //         left_hash: [
    //             192, 103, 168, 209, 21, 23, 137, 121, 173, 138, 179, 199, 124, 163, 200, 22, 101,
    //             85, 103, 102, 253, 118, 15, 195, 248, 223, 181, 228, 63, 234, 156, 135,
    //         ],
    //         version: 0,
    //     };

    //     let z = InnerNode {
    //         right_node: Some(Box::new(Node::Inner(y))),
    //         left_node: Some(Box::new(Node::Leaf(LeafNode {
    //             key: vec![18],
    //             value: vec![3, 2, 2],
    //             version: 0,
    //         }))),
    //         key: vec![19],
    //         height: 3,
    //         size: 4,
    //         left_hash: [
    //             121, 226, 107, 73, 123, 135, 165, 82, 94, 53, 112, 50, 126, 200, 252, 137, 235, 87,
    //             205, 133, 96, 202, 94, 222, 39, 138, 231, 198, 189, 196, 49, 196,
    //         ],
    //         right_hash: [
    //             13, 181, 53, 227, 140, 38, 242, 22, 94, 152, 94, 71, 0, 89, 35, 122, 129, 85, 55,
    //             190, 253, 226, 35, 230, 65, 214, 244, 35, 69, 39, 223, 90,
    //         ],
    //         version: 0,
    //     };

    //     let mut z = Node::Inner(z);

    //     let db = MemDB::new();
    //     Tree::left_rotate(&mut z, 0, &NodeDB::new(db, 100)).unwrap();

    //     let hash = z.hash();
    //     let expected = [
    //         221, 58, 23, 0, 25, 206, 49, 41, 174, 43, 173, 118, 31, 30, 46, 172, 195, 159, 69, 125,
    //         238, 68, 72, 17, 217, 148, 126, 112, 95, 17, 115, 160,
    //     ];
    //     assert_eq!(hash, expected)
    // }

    // #[test]
    // fn set_equal_leaf_works() {
    //     let db = MemDB::new();
    //     let mut tree = Tree::new(db, None, 100).unwrap();
    //     tree.set(vec![1], vec![2]);
    //     tree.set(vec![1], vec![3]);

    //     let hash = tree.root_hash();
    //     let expected = [
    //         146, 114, 60, 233, 157, 240, 49, 35, 57, 65, 154, 83, 84, 160, 123, 45, 153, 137, 215,
    //         139, 195, 141, 74, 219, 86, 182, 75, 239, 223, 87, 133, 81,
    //     ];
    //     assert_eq!(hash, expected)
    // }

    // #[test]
    // fn set_less_than_leaf_works() {
    //     let db = MemDB::new();
    //     let mut tree = Tree::new(db, None, 100).unwrap();
    //     tree.set(vec![3], vec![2]);
    //     tree.set(vec![1], vec![3]);

    //     let hash = tree.root_hash();
    //     let expected = [
    //         197, 117, 162, 213, 61, 146, 253, 165, 111, 237, 42, 95, 186, 76, 202, 167, 174, 187,
    //         19, 6, 150, 29, 243, 41, 209, 142, 80, 45, 32, 9, 235, 24,
    //     ];
    //     assert_eq!(hash, expected)
    // }

    // #[test]
    // fn set_greater_than_leaf_works() {
    //     let db = MemDB::new();
    //     let mut tree = Tree::new(db, None, 100).unwrap();
    //     tree.set(vec![1], vec![2]);
    //     tree.set(vec![3], vec![3]);

    //     let hash = tree.root_hash();
    //     let expected = [
    //         27, 213, 240, 14, 167, 98, 231, 104, 130, 46, 40, 228, 172, 2, 149, 149, 32, 10, 198,
    //         129, 179, 18, 29, 182, 227, 231, 178, 29, 160, 69, 142, 244,
    //     ];
    //     assert_eq!(hash, expected)
    // }

    // #[test]
    // fn repeated_set_works() {
    //     let db = MemDB::new();
    //     let mut tree = Tree::new(db, None, 100).unwrap();
    //     tree.set(b"alice".to_vec(), b"abc".to_vec());
    //     tree.set(b"bob".to_vec(), b"123".to_vec());
    //     tree.set(b"c".to_vec(), b"1".to_vec());
    //     tree.set(b"q".to_vec(), b"1".to_vec());

    //     let expected = [
    //         202, 52, 159, 10, 210, 166, 72, 207, 248, 190, 60, 114, 172, 147, 84, 27, 120, 202,
    //         189, 127, 230, 108, 58, 127, 251, 149, 9, 33, 87, 249, 158, 138,
    //     ];

    //     assert_eq!(expected, tree.root_hash());
    // }

    // #[test]
    // fn save_version_works() {
    //     let db = MemDB::new();
    //     let mut tree = Tree::new(db, None, 100).unwrap();
    //     tree.set(b"alice".to_vec(), b"abc".to_vec());
    //     tree.set(b"bob".to_vec(), b"123".to_vec());
    //     tree.set(b"c".to_vec(), b"1".to_vec());
    //     tree.set(b"q".to_vec(), b"1".to_vec());

    //     tree.save_version().unwrap();
    //     tree.save_version().unwrap();
    //     tree.set(b"qwerty".to_vec(), b"312".to_vec());
    //     tree.set(b"-32".to_vec(), b"gamma".to_vec());
    //     tree.save_version().unwrap();
    //     tree.set(b"alice".to_vec(), b"123".to_vec());
    //     tree.save_version().unwrap();

    //     let expected = [
    //         37, 155, 233, 229, 243, 173, 29, 241, 235, 234, 85, 10, 36, 129, 53, 79, 77, 11, 29,
    //         118, 201, 233, 133, 60, 78, 187, 37, 81, 42, 96, 105, 150,
    //     ];

    //     assert_eq!(expected, tree.root_hash());
    // }

    // #[test]
    // fn get_works() {
    //     let db = MemDB::new();
    //     let mut tree = Tree::new(db, None, 100).unwrap();
    //     tree.set(b"alice".to_vec(), b"abc".to_vec());
    //     tree.set(b"bob".to_vec(), b"123".to_vec());
    //     tree.set(b"c".to_vec(), b"1".to_vec());
    //     tree.set(b"q".to_vec(), b"1".to_vec());

    //     assert_eq!(tree.get(b"alice"), Some(String::from("abc").into()));
    //     assert_eq!(tree.get(b"bob"), Some(String::from("123").into()));
    //     assert_eq!(tree.get(b"c"), Some(String::from("1").into()));
    //     assert_eq!(tree.get(b"q"), Some(String::from("1").into()));
    //     assert_eq!(tree.get(b"house"), None);
    // }

    // #[test]
    // fn scenario_works() {
    //     let db = MemDB::new();
    //     let mut tree = Tree::new(db, None, 100).unwrap();
    //     tree.set(vec![0, 117, 97, 116, 111, 109], vec![51, 52]);
    //     tree.set(
    //         vec![
    //             2, 20, 129, 58, 194, 42, 97, 73, 22, 85, 226, 120, 106, 224, 209, 39, 214, 153, 11,
    //             251, 251, 222, 117, 97, 116, 111, 109,
    //         ],
    //         vec![10, 5, 117, 97, 116, 111, 109, 18, 2, 51, 52],
    //     );

    //     tree.save_version().unwrap();
    //     tree.save_version().unwrap();
    //     tree.save_version().unwrap();
    //     tree.save_version().unwrap();
    //     tree.save_version().unwrap();
    //     tree.save_version().unwrap();
    //     tree.save_version().unwrap();

    //     tree.set(
    //         vec![
    //             2, 20, 59, 214, 51, 187, 112, 177, 248, 133, 197, 68, 36, 228, 124, 164, 14, 68,
    //             72, 143, 236, 46, 117, 97, 116, 111, 109,
    //         ],
    //         vec![10, 5, 117, 97, 116, 111, 109, 18, 2, 49, 48],
    //     );
    //     tree.set(
    //         vec![
    //             2, 20, 129, 58, 194, 42, 97, 73, 22, 85, 226, 120, 106, 224, 209, 39, 214, 153, 11,
    //             251, 251, 222, 117, 97, 116, 111, 109,
    //         ],
    //         vec![10, 5, 117, 97, 116, 111, 109, 18, 2, 50, 51],
    //     );
    //     tree.set(
    //         vec![
    //             2, 20, 241, 130, 150, 118, 219, 87, 118, 130, 233, 68, 252, 52, 147, 212, 81, 182,
    //             127, 243, 226, 159, 117, 97, 116, 111, 109,
    //         ],
    //         vec![10, 5, 117, 97, 116, 111, 109, 18, 1, 49],
    //     );

    //     let expected = [
    //         34, 215, 64, 141, 118, 237, 192, 198, 47, 22, 34, 81, 0, 146, 145, 66, 182, 59, 101,
    //         145, 99, 187, 82, 49, 149, 36, 196, 63, 37, 42, 171, 124,
    //     ];

    //     let (hash, version) = tree.save_version().unwrap();

    //     assert_eq!((expected, 8), (hash, version));
    // }

    // #[test]
    // fn bounded_range_works() {
    //     let db = MemDB::new();
    //     let mut tree = Tree::new(db, None, 100).unwrap();
    //     tree.set(b"1".to_vec(), b"abc1".to_vec());

    //     tree.set(b"2".to_vec(), b"abc2".to_vec());
    //     tree.set(b"3".to_vec(), b"abc3".to_vec());
    //     tree.set(b"4".to_vec(), b"abc4".to_vec());
    //     tree.set(b"5".to_vec(), b"abc5".to_vec());
    //     tree.set(b"6".to_vec(), b"abc6".to_vec());
    //     tree.set(b"7".to_vec(), b"abc7".to_vec());

    //     // [,)
    //     let start = b"3".to_vec();
    //     let stop = b"6".to_vec();
    //     let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = tree.range(start..stop).collect();
    //     let expected_pairs = vec![
    //         (b"3".to_vec(), b"abc3".to_vec()),
    //         (b"4".to_vec(), b"abc4".to_vec()),
    //         (b"5".to_vec(), b"abc5".to_vec()),
    //     ];

    //     assert_eq!(expected_pairs.len(), got_pairs.len());
    //     assert!(expected_pairs.into_iter().all(|e| {
    //         let cmp = (e.0, e.1);
    //         got_pairs.contains(&cmp)
    //     }));

    //     // [,]
    //     let start = b"3".to_vec();
    //     let stop = b"6".to_vec();
    //     let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = tree.range(start..=stop).collect();
    //     let expected_pairs = vec![
    //         (b"3".to_vec(), b"abc3".to_vec()),
    //         (b"4".to_vec(), b"abc4".to_vec()),
    //         (b"5".to_vec(), b"abc5".to_vec()),
    //         (b"6".to_vec(), b"abc6".to_vec()),
    //     ];

    //     assert_eq!(expected_pairs.len(), got_pairs.len());
    //     assert!(expected_pairs.into_iter().all(|e| {
    //         let cmp = (e.0, e.1);
    //         got_pairs.contains(&cmp)
    //     }));

    //     // (,)
    //     let start = b"3".to_vec();
    //     let stop = b"6".to_vec();
    //     let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = tree
    //         .range((Bound::Excluded(start), Bound::Excluded(stop)))
    //         .collect();
    //     let expected_pairs = vec![
    //         (b"4".to_vec(), b"abc4".to_vec()),
    //         (b"5".to_vec(), b"abc5".to_vec()),
    //     ];

    //     assert_eq!(expected_pairs.len(), got_pairs.len());
    //     assert!(expected_pairs.into_iter().all(|e| {
    //         let cmp = (e.0, e.1);
    //         got_pairs.contains(&cmp)
    //     }));
    // }

    // #[test]
    // fn full_range_unique_keys_works() {
    //     let db = MemDB::new();
    //     let mut tree = Tree::new(db, None, 100).unwrap();
    //     tree.set(b"alice".to_vec(), b"abc".to_vec());
    //     tree.set(b"bob".to_vec(), b"123".to_vec());
    //     tree.set(b"c".to_vec(), b"1".to_vec());
    //     tree.set(b"q".to_vec(), b"1".to_vec());
    //     let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = tree.range(..).collect();

    //     let expected_pairs = vec![
    //         (b"alice".to_vec(), b"abc".to_vec()),
    //         (b"c".to_vec(), b"1".to_vec()),
    //         (b"q".to_vec(), b"1".to_vec()),
    //         (b"bob".to_vec(), b"123".to_vec()),
    //     ];

    //     assert_eq!(expected_pairs.len(), got_pairs.len());
    //     assert!(expected_pairs.into_iter().all(|e| {
    //         let cmp = (e.0, e.1);
    //         got_pairs.contains(&cmp)
    //     }));
    // }

    // #[test]
    // fn full_range_duplicate_keys_works() {
    //     let db = MemDB::new();
    //     let mut tree = Tree::new(db, None, 100).unwrap();
    //     tree.set(b"alice".to_vec(), b"abc".to_vec());
    //     tree.set(b"alice".to_vec(), b"abc".to_vec());
    //     let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = tree.range(..).collect();

    //     let expected_pairs = vec![(b"alice".to_vec(), b"abc".to_vec())];

    //     assert_eq!(expected_pairs.len(), got_pairs.len());
    //     assert!(expected_pairs.into_iter().all(|e| {
    //         let cmp = (e.0, e.1);
    //         got_pairs.contains(&cmp)
    //     }));
    // }

    // #[test]
    // fn empty_tree_range_works() {
    //     let db = MemDB::new();
    //     let tree = Tree::new(db, None, 100).unwrap();
    //     let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = tree.range(..).collect();

    //     let expected_pairs: Vec<(Vec<u8>, Vec<u8>)> = vec![];

    //     assert_eq!(expected_pairs.len(), got_pairs.len());
    //     assert!(expected_pairs.into_iter().all(|e| {
    //         let cmp = (e.0, e.1);
    //         got_pairs.contains(&cmp)
    //     }));
    // }

    // #[test]
    // fn serialize_deserialize_inner_works() {
    //     let orig_node = Node::Inner(InnerNode {
    //         left_node: None,
    //         right_node: None,
    //         key: vec![19],
    //         height: 3,
    //         size: 4,
    //         left_hash: [
    //             121, 226, 107, 73, 123, 135, 165, 82, 94, 53, 112, 50, 126, 200, 252, 137, 235, 87,
    //             205, 133, 96, 202, 94, 222, 39, 138, 231, 198, 189, 196, 49, 196,
    //         ],
    //         right_hash: [
    //             13, 181, 53, 227, 140, 38, 242, 22, 94, 152, 94, 71, 0, 89, 35, 122, 129, 85, 55,
    //             190, 253, 226, 35, 230, 65, 214, 244, 35, 69, 39, 223, 90,
    //         ],
    //         version: 0,
    //     });

    //     let node_bytes = orig_node.serialize();
    //     assert_eq!(
    //         node_bytes,
    //         [
    //             3, 4, 0, 1, 19, 32, 121, 226, 107, 73, 123, 135, 165, 82, 94, 53, 112, 50, 126,
    //             200, 252, 137, 235, 87, 205, 133, 96, 202, 94, 222, 39, 138, 231, 198, 189, 196,
    //             49, 196, 32, 13, 181, 53, 227, 140, 38, 242, 22, 94, 152, 94, 71, 0, 89, 35, 122,
    //             129, 85, 55, 190, 253, 226, 35, 230, 65, 214, 244, 35, 69, 39, 223, 90
    //         ]
    //     );
    //     let deserialized_node = Node::deserialize(node_bytes).unwrap();
    //     assert_eq!(deserialized_node, orig_node);
    // }

    // #[test]
    // fn serialize_deserialize_leaf_works() {
    //     let orig_node = Node::Leaf(LeafNode {
    //         key: vec![19],
    //         value: vec![1, 2, 3],
    //         version: 0,
    //     });

    //     let node_bytes = orig_node.serialize();
    //     assert_eq!(node_bytes, [0, 1, 0, 1, 19, 3, 1, 2, 3]);
    //     let deserialized_node = Node::deserialize(node_bytes).unwrap();
    //     assert_eq!(deserialized_node, orig_node);
    // }

    // /// Testing that a previous bug has been fixed
    // #[test]
    // fn bug_scenario_works() {
    //     let db = MemDB::new();
    //     let mut tree = Tree::new(db, None, 100).unwrap();
    //     tree.set(vec![0], vec![8, 244, 162, 237, 1]);
    //     tree.save_version().unwrap();
    //     tree.set(vec![0], vec![8, 133, 164, 237, 1]);
    //     tree.save_version().unwrap();
    //     tree.set(vec![0], vec![8, 133, 164, 237, 1]);
    //     tree.save_version().unwrap();
    //     tree.set(vec![0], vec![8, 135, 164, 237, 1]);
    //     tree.set(
    //         vec![
    //             1, 173, 86, 59, 0, 0, 0, 0, 0, 1, 129, 58, 194, 42, 97, 73, 22, 85, 226, 120, 106,
    //             224, 209, 39, 214, 153, 11, 251, 251, 222,
    //         ],
    //         vec![
    //             10, 45, 99, 111, 115, 109, 111, 115, 49, 115, 121, 97, 118, 121, 50, 110, 112, 102,
    //             121, 116, 57, 116, 99, 110, 99, 100, 116, 115, 100, 122, 102, 55, 107, 110, 121,
    //             57, 108, 104, 55, 55, 55, 112, 97, 104, 117, 117, 120, 16, 173, 173, 237, 1, 24, 1,
    //             34, 3, 1, 2, 3,
    //         ],
    //     );
    //     tree.set(
    //         vec![2, 173, 86, 59, 0, 0, 0, 0, 0, 1],
    //         vec![8, 173, 173, 237, 1, 16, 1],
    //     );
    //     tree.save_version().unwrap();
    //     tree.set(vec![0], vec![8, 137, 164, 237, 1]);
    //     tree.set(
    //         vec![
    //             1, 173, 86, 59, 0, 0, 0, 0, 0, 1, 133, 145, 191, 185, 82, 168, 56, 30, 164, 88, 69,
    //             0, 206, 225, 190, 214, 210, 36, 231, 69,
    //         ],
    //         vec![
    //             10, 45, 99, 111, 115, 109, 111, 115, 49, 115, 107, 103, 109, 108, 119, 50, 106, 52,
    //             113, 117, 112, 97, 102, 122, 99, 103, 53, 113, 118, 97, 99, 100, 55, 54, 109, 102,
    //             122, 102, 101, 54, 57, 108, 97, 48, 104, 120, 122, 16, 173, 173, 237, 1, 24, 1, 34,
    //             3, 1, 2, 3,
    //         ],
    //     );
    //     tree.set(
    //         vec![2, 173, 86, 59, 0, 0, 0, 0, 0, 1],
    //         vec![8, 173, 173, 237, 1, 16, 1],
    //     );
    //     tree.save_version().unwrap();
    //     tree.set(vec![0], vec![8, 138, 164, 237, 1]);
    //     tree.set(
    //         vec![
    //             1, 174, 86, 59, 0, 0, 0, 0, 0, 1, 133, 145, 191, 185, 82, 168, 56, 30, 164, 88, 69,
    //             0, 206, 225, 190, 214, 210, 36, 231, 69,
    //         ],
    //         vec![
    //             10, 45, 99, 111, 115, 109, 111, 115, 49, 115, 107, 103, 109, 108, 119, 50, 106, 52,
    //             113, 117, 112, 97, 102, 122, 99, 103, 53, 113, 118, 97, 99, 100, 55, 54, 109, 102,
    //             122, 102, 101, 54, 57, 108, 97, 48, 104, 120, 122, 16, 174, 173, 237, 1, 24, 1, 34,
    //             3, 1, 2, 3,
    //         ],
    //     );
    //     tree.set(
    //         vec![2, 174, 86, 59, 0, 0, 0, 0, 0, 1],
    //         vec![8, 174, 173, 237, 1, 16, 1],
    //     );
    //     tree.save_version().unwrap();
    //     tree.set(vec![0], vec![8, 140, 164, 237, 1]);
    //     tree.save_version().unwrap();
    //     tree.set(vec![0], vec![8, 142, 164, 237, 1]);

    //     tree.set(
    //         vec![
    //             1, 174, 86, 59, 0, 0, 0, 0, 0, 1, 129, 58, 194, 42, 97, 73, 22, 85, 226, 120, 106,
    //             224, 209, 39, 214, 153, 11, 251, 251, 222,
    //         ],
    //         vec![
    //             10, 45, 99, 111, 115, 109, 111, 115, 49, 115, 121, 97, 118, 121, 50, 110, 112, 102,
    //             121, 116, 57, 116, 99, 110, 99, 100, 116, 115, 100, 122, 102, 55, 107, 110, 121,
    //             57, 108, 104, 55, 55, 55, 112, 97, 104, 117, 117, 120, 16, 174, 173, 237, 1, 24, 1,
    //             34, 3, 1, 2, 3,
    //         ],
    //     );

    //     tree.save_version().unwrap();

    //     let expected = [
    //         136, 164, 1, 21, 163, 66, 127, 238, 197, 107, 178, 152, 75, 8, 254, 220, 62, 141, 140,
    //         212, 4, 23, 213, 249, 34, 96, 132, 172, 166, 207, 48, 17,
    //     ];

    //     assert!(is_consistent(tree.root.clone().unwrap(), &tree.node_db));
    //     assert_eq!(expected, tree.root_hash());
    // }

    // /// Testing that a previous bug has been fixed
    // #[test]
    // fn bug_scenario_2_works() {
    //     let db = MemDB::new();
    //     let mut tree = Tree::new(db, None, 100).unwrap();
    //     tree.set(
    //         vec![
    //             0, 0, 0, 0, 0, 0, 0, 0, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58,
    //         ],
    //         vec![
    //             0, 0, 0, 0, 0, 0, 0, 0, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58,
    //         ],
    //     );

    //     tree.set(
    //         vec![
    //             0, 0, 0, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //         ],
    //         vec![
    //             0, 0, 0, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //         ],
    //     );
    //     tree.set(
    //         vec![
    //             0, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 0, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //         ],
    //         vec![
    //             0, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 0, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //         ],
    //     );

    //     tree.set(
    //         vec![
    //             0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //         ],
    //         vec![
    //             0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //         ],
    //     );

    //     tree.save_version().unwrap();

    //     let expected = [
    //         161, 141, 64, 164, 190, 244, 170, 230, 150, 211, 45, 54, 92, 136, 170, 253, 7, 176,
    //         179, 212, 27, 116, 84, 160, 78, 92, 155, 245, 98, 143, 221, 105,
    //     ];

    //     let root = tree.root.as_ref().unwrap();

    //     assert!(is_consistent(root, &tree.node_db));
    //     assert_eq!(expected, tree.root_hash());
    // }

    // /// Checks if left/right hash matches the left/right node hash for every inner node in a tree
    // fn is_consistent<T: Database, N>(root: N, node_db: &NodeDB<T>) -> bool
    // where
    //     N: Borrow<Node>,
    // {
    //     match root.borrow() {
    //         Node::Inner(node) => {
    //             let left_node = match &node.left_node {
    //                 Some(left_node) => *left_node.clone(),
    //                 None => node_db
    //                     .get_node(&node.left_hash)
    //                     .expect("node db should contain all nodes"),
    //             };

    //             let right_node = match &node.right_node {
    //                 Some(right_node) => *right_node.clone(),
    //                 None => node_db
    //                     .get_node(&node.right_hash)
    //                     .expect("node db should contain all nodes"),
    //             };

    //             if left_node.hash() != node.left_hash {
    //                 return false;
    //             }

    //             if right_node.hash() != node.right_hash {
    //                 return false;
    //             }

    //             if !is_consistent(left_node, node_db) {
    //                 return false;
    //             }

    //             if !is_consistent(right_node, node_db) {
    //                 return false;
    //             }

    //             true
    //         }
    //         Node::Leaf(_) => true,
    //     }
    // }
}
