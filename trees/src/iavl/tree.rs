use std::{
    cmp,
    collections::BTreeSet,
    mem,
    ops::{Bound, RangeBounds},
    sync::Arc,
};

use database::DB;
use integer_encoding::VarInt;
use sha2::{Digest, Sha256};

use crate::{error::Error, merkle::EMPTY_HASH};

use super::node_db::NodeDB;

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct InnerNode {
    left_node: Option<Arc<Node>>, // None means value is the same as what's in the DB
    right_node: Option<Arc<Node>>,
    key: Vec<u8>,
    height: u8,
    size: u32, // number of leaf nodes in this node's subtrees
    left_hash: [u8; 32],
    right_hash: [u8; 32],
    version: u32,
}

impl InnerNode {
    fn get_mut_left_node<T: DB>(&mut self, node_db: &NodeDB<T>) -> &mut Node {
        if self.left_node.is_none() {
            let node = node_db
                .get_node(&self.left_hash)
                .expect("this node should be in the DB");

            self.left_node = Some(Arc::new(node));
        }

        match &mut self.left_node {
            Some(node) => {
                Arc::get_mut(node).expect("there are no other Arc pointers to this allocation")
            }
            None => panic!("it can't be None given the block above"),
        }
    }

    fn get_left_node<T: DB>(&self, node_db: &NodeDB<T>) -> Arc<Node> {
        match &self.left_node {
            Some(node) => return node.clone(),
            None => Arc::new(
                node_db
                    .get_node(&self.left_hash)
                    .expect("this node should be in the DB"),
            ),
        }
    }

    fn get_mut_right_node<T: DB>(&mut self, node_db: &NodeDB<T>) -> &mut Node {
        if self.right_node.is_none() {
            let node = node_db
                .get_node(&self.right_hash)
                .expect("this node should be in the DB");

            self.right_node = Some(Arc::new(node));
        }

        match &mut self.right_node {
            Some(node) => {
                Arc::get_mut(node).expect("there are no other Arc pointers to this allocation")
            }
            None => panic!("it can't be None given the block above"),
        }
    }

    fn get_right_node<T: DB>(&self, node_db: &NodeDB<T>) -> Arc<Node> {
        match &self.right_node {
            Some(node) => return node.clone(),
            None => Arc::new(
                node_db
                    .get_node(&self.right_hash)
                    .expect("this node should be in the DB"),
            ),
        }
    }

    fn get_balance_factor<T: DB>(&self, node_db: &NodeDB<T>) -> i16 {
        let left_height: i16 = self.get_left_node(node_db).get_height().into();
        let right_height: i16 = self.get_right_node(node_db).get_height().into();
        left_height - right_height
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct LeafNode {
    key: Vec<u8>,
    value: Vec<u8>,
    version: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Node {
    Leaf(LeafNode),
    Inner(InnerNode),
}

impl Default for Node {
    fn default() -> Self {
        Node::Leaf(Default::default())
    }
}

impl Node {
    pub fn get_key(&self) -> &Vec<u8> {
        match self {
            Node::Leaf(leaf) => &leaf.key,
            Node::Inner(inner) => &inner.key,
        }
    }

    pub fn get_height(&self) -> u8 {
        match self {
            Node::Leaf(leaf) => 0,
            Node::Inner(inner) => inner.height,
        }
    }

    pub fn new_leaf(key: Vec<u8>, value: Vec<u8>, version: u32) -> Node {
        return Node::Leaf(LeafNode {
            key,
            value,
            version,
        });
    }

    pub fn hash(&self) -> [u8; 32] {
        let serialized = self.hash_serialize();
        Sha256::digest(serialized).into()
    }

    fn hash_serialize(&self) -> Vec<u8> {
        match &self {
            Node::Leaf(node) => {
                // NOTE: i64 is used here for parameters for compatibility wih cosmos
                let height: i64 = 0;
                let size: i64 = 1;
                let version: i64 = node.version.into();
                let hashed_value = Sha256::digest(&node.value);

                let mut serialized = height.encode_var_vec();
                serialized.extend(size.encode_var_vec());
                serialized.extend(version.encode_var_vec());
                serialized.extend(encode_bytes(&node.key));
                serialized.extend(encode_bytes(&hashed_value));

                return serialized;
            }
            Node::Inner(node) => {
                // NOTE: i64 is used here for parameters for compatibility wih cosmos
                let height: i64 = node.height.into();
                let size: i64 = node.size.into();
                let version: i64 = node.version.into();

                let mut node_bytes = height.encode_var_vec();
                node_bytes.extend(size.encode_var_vec());
                node_bytes.extend(version.encode_var_vec());
                node_bytes.extend(encode_bytes(&node.left_hash));
                node_bytes.extend(encode_bytes(&node.right_hash));

                return node_bytes;
            }
        }
    }

    pub(crate) fn deserialize(bytes: Vec<u8>) -> Result<Self, Error> {
        let (height, mut n) = u8::decode_var(&bytes).ok_or(Error::NodeDeserialize)?;
        let (size, ns) = u32::decode_var(&bytes[n..]).ok_or(Error::NodeDeserialize)?;
        n += ns;
        let (version, nv) = u32::decode_var(&bytes[n..]).ok_or(Error::NodeDeserialize)?;
        n += nv;
        let (key, nk) = decode_bytes(&bytes[n..])?;
        n += nk;

        if height == 0 {
            // leaf node
            let (value, _) = decode_bytes(&bytes[n..])?;

            Ok(Node::Leaf(LeafNode {
                key,
                value,
                version,
            }))
        } else {
            // inner node
            let (left_hash, nl) = decode_bytes(&bytes[n..])?;
            n += nl;

            let (right_hash, _) = decode_bytes(&bytes[n..])?;

            Ok(Node::Inner(InnerNode {
                left_node: None,
                right_node: None,
                key,
                height,
                size,
                left_hash: left_hash.try_into().map_err(|_| Error::NodeDeserialize)?,
                right_hash: right_hash.try_into().map_err(|_| Error::NodeDeserialize)?,
                version,
            }))
        }
    }

    fn get_size(&self) -> u32 {
        match &self {
            Node::Leaf(_) => 1,
            Node::Inner(n) => n.size,
        }
    }
}

#[derive(Debug)]
pub struct Tree<T>
where
    T: DB,
{
    root: Option<Arc<Node>>,
    node_db: NodeDB<T>,
    version: u32,
    versions: BTreeSet<u32>,
}

impl<T> Tree<T>
where
    T: DB,
{
    pub fn new(db: T, target_version: Option<u32>) -> Result<Tree<T>, Error> {
        let node_db = NodeDB::new(db);
        let versions = node_db.get_versions();

        if let Some(target_version) = target_version {
            let root = node_db
                .get_root(target_version)
                .ok_or(Error::VersionNotFound)?;

            Ok(Tree {
                root: Some(Arc::new(root)),
                version: target_version,
                node_db,
                versions,
            })
        } else {
            // use the latest version available
            if let Some(latest_version) = versions.last() {
                Ok(Tree {
                    root: node_db.get_root(*latest_version).map(|n| Arc::new(n)),
                    version: *latest_version,
                    node_db,
                    versions,
                })
            } else {
                Ok(Tree {
                    root: None,
                    version: 0,
                    node_db,
                    versions,
                })
            }
        }
    }

    pub fn save_version(&mut self) -> ([u8; 32], u32) {
        self.version += 1;
        (self.root_hash(), self.version)
    }

    pub fn root_hash(&self) -> [u8; 32] {
        match &self.root {
            Some(root) => root.hash(),
            None => EMPTY_HASH,
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        match &self.root {
            Some(root) => Self::recursive_get(root.clone(), key, &self.node_db),
            None => None,
        }
    }

    fn recursive_get<'a>(node: Arc<Node>, key: &[u8], node_db: &NodeDB<T>) -> Option<Vec<u8>> {
        match &*node {
            Node::Leaf(leaf) => {
                if leaf.key == key {
                    return Some(leaf.value.clone());
                } else {
                    return None;
                }
            }
            Node::Inner(node) => {
                if key < &node.key {
                    let left_node = node.get_left_node(node_db);
                    return Self::recursive_get(left_node, key, node_db);
                } else {
                    let right_node = node.get_right_node(node_db);
                    return Self::recursive_get(right_node, key, node_db);
                }
            }
        }
    }

    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
        match &mut self.root {
            Some(root) => Self::recursive_set(
                Arc::get_mut(root).expect("there are no other Arc pointers to this allocation"),
                key,
                value,
                self.version + 1,
                &mut self.node_db,
            ),
            None => {
                self.root = Some(Arc::new(Node::Leaf(LeafNode {
                    key,
                    value,
                    version: self.version + 1,
                })));
            }
        };
    }

    fn recursive_set(
        mut node: &mut Node,
        key: Vec<u8>,
        value: Vec<u8>,
        version: u32,
        node_db: &mut NodeDB<T>,
    ) {
        match &mut node {
            Node::Leaf(leaf_node) => {
                match key.cmp(&leaf_node.key) {
                    cmp::Ordering::Less => {
                        let left_node = Node::new_leaf(key, value, version);
                        let left_hash = left_node.hash();
                        let right_node = Node::Leaf(leaf_node.clone());
                        let right_hash = right_node.hash();

                        *node = Node::Inner(InnerNode {
                            key: leaf_node.key.clone(),
                            left_node: Some(Arc::new(left_node)),
                            right_node: Some(Arc::new(right_node)),
                            height: 1,
                            size: 2,
                            version,
                            left_hash,
                            right_hash,
                        });
                        return;
                    }
                    cmp::Ordering::Equal => {
                        leaf_node.value = value;
                        leaf_node.version = version;
                        return;
                    }
                    cmp::Ordering::Greater => {
                        let right_node = Node::new_leaf(key.clone(), value, version);
                        let right_hash = right_node.hash();
                        let left_subtree = node.clone();
                        let left_hash = left_subtree.hash();

                        *node = Node::Inner(InnerNode {
                            key,
                            left_node: Some(Arc::new(left_subtree)),
                            right_node: Some(Arc::new(right_node)),
                            height: 1,
                            size: 2,
                            left_hash,
                            right_hash,
                            version,
                        });
                        return;
                    }
                };
            }
            Node::Inner(root_node) => {
                // Perform normal BST
                if key < root_node.key {
                    Self::recursive_set(
                        root_node.get_mut_left_node(node_db),
                        key.clone(),
                        value,
                        version,
                        node_db,
                    );
                    root_node.left_hash = root_node.get_left_node(node_db).hash();
                } else {
                    Self::recursive_set(
                        root_node.get_mut_right_node(node_db),
                        key.clone(),
                        value,
                        version,
                        node_db,
                    );
                    root_node.right_hash = root_node.get_right_node(node_db).hash();
                }

                // Update height + size + version
                root_node.height = 1 + cmp::max(
                    root_node.get_left_node(node_db).get_height(),
                    root_node.get_right_node(node_db).get_height(),
                );

                root_node.size = root_node.get_left_node(node_db).get_size()
                    + root_node.get_right_node(node_db).get_size();

                root_node.version = version;

                // If the tree is unbalanced then try out the usual four cases
                let balance_factor = root_node.get_balance_factor(node_db);
                if balance_factor > 1 {
                    let left_node = root_node.get_mut_left_node(node_db);

                    if &key < left_node.get_key() {
                        // Case 1 - Right
                        Self::right_rotate(node, version, node_db)
                            .expect("Given the imbalance, expect rotation to always succeed");
                    } else {
                        // Case 2 - Left Right
                        Self::left_rotate(node, version, node_db)
                            .expect("Given the imbalance, expect rotation to always succeed");
                        Self::right_rotate(node, version, node_db)
                            .expect("Given the imbalance, expect rotation to always succeed");
                    }
                } else if balance_factor < -1 {
                    let right_node = root_node.get_mut_right_node(node_db);

                    if &key > right_node.get_key() {
                        // Case 3 - Left
                        Self::left_rotate(node, version, node_db)
                            .expect("Given the imbalance, expect rotation to always succeed");
                    } else {
                        // Case 4 - Right Left
                        Self::right_rotate(right_node, version, node_db)
                            .expect("Given the imbalance, expect rotation to always succeed");
                        Self::left_rotate(node, version, node_db)
                            .expect("Given the imbalance, expect rotation to always succeed");
                    }
                }
            }
        };
    }

    fn right_rotate(node: &mut Node, version: u32, node_db: &NodeDB<T>) -> Result<(), Error> {
        if let Node::Inner(z) = node {
            let mut z = mem::take(z);
            let y = mem::take(z.get_mut_left_node(node_db));

            let mut y = match y {
                Node::Inner(y) => y,
                Node::Leaf(_) => return Err(Error::RotateError),
            };

            let t3 = y.right_node;

            // Perform rotation on z and update height and hash
            z.left_node = t3;
            z.height = 1 + cmp::max(
                z.get_left_node(node_db).get_height(),
                z.get_right_node(node_db).get_height(),
            );
            z.size = z.get_left_node(node_db).get_size() + z.get_right_node(node_db).get_size();
            z.version = version;
            z.left_hash = y.right_hash;
            let z = Node::Inner(z);

            // Perform rotation on y, update hash and update height
            y.right_hash = z.hash();
            y.right_node = Some(Arc::new(z));
            y.height = 1 + cmp::max(
                y.get_left_node(node_db).get_height(),
                y.get_right_node(node_db).get_height(),
            );
            y.size = y.get_left_node(node_db).get_size() + y.get_right_node(node_db).get_size();
            y.version = version;

            *node = Node::Inner(y);

            return Ok(());
        } else {
            // Can't rotate a leaf node
            return Err(Error::RotateError);
        }
    }

    fn left_rotate(node: &mut Node, version: u32, node_db: &NodeDB<T>) -> Result<(), Error> {
        if let Node::Inner(z) = node {
            let mut z = mem::take(z);
            let y = mem::take(z.get_mut_right_node(node_db));

            let mut y = match y {
                Node::Inner(y) => y,
                Node::Leaf(_) => return Err(Error::RotateError),
            };

            let t2 = y.left_node;

            // Perform rotation on z and update height and hash
            z.right_node = t2;
            z.height = 1 + cmp::max(
                z.get_left_node(node_db).get_height(),
                z.get_right_node(node_db).get_height(),
            );
            z.size = z.get_left_node(node_db).get_size() + z.get_right_node(node_db).get_size();
            z.version = version;
            z.right_hash = y.left_hash;
            let z = Node::Inner(z);

            // Perform rotation on y, update hash and update height
            y.left_hash = z.hash();
            y.left_node = Some(Arc::new(z));
            y.height = 1 + cmp::max(
                y.get_left_node(node_db).get_height(),
                y.get_right_node(node_db).get_height(),
            );
            y.size = y.get_left_node(node_db).get_size() + y.get_right_node(node_db).get_size();
            y.version = version;

            *node = Node::Inner(y);

            return Ok(());
        } else {
            // Can't rotate a leaf node
            return Err(Error::RotateError);
        }
    }

    pub fn range<R>(&self, range: R) -> Range<R, T>
    where
        R: RangeBounds<Vec<u8>>,
    {
        match &self.root {
            Some(root) => Range {
                range,
                delayed_nodes: vec![root.clone()],
                node_db: &self.node_db,
            },
            None => Range {
                range,
                delayed_nodes: vec![],
                node_db: &&self.node_db,
            },
        }
    }
}

pub struct Range<'a, R: RangeBounds<Vec<u8>>, T>
where
    T: DB,
{
    range: R,
    delayed_nodes: Vec<Arc<Node>>,
    node_db: &'a NodeDB<T>,
}

impl<'a, T: RangeBounds<Vec<u8>>, R: DB> Range<'a, T, R> {
    fn traverse(&mut self) -> Option<(Vec<u8>, Vec<u8>)> {
        let node = self.delayed_nodes.pop()?;

        let after_start = match self.range.start_bound() {
            Bound::Included(l) => node.get_key() > l,
            Bound::Excluded(l) => node.get_key() > l,
            Bound::Unbounded => true,
        };

        let before_end = match self.range.end_bound() {
            Bound::Included(u) => node.get_key() <= u,
            Bound::Excluded(u) => node.get_key() < u,
            Bound::Unbounded => true,
        };

        match &*node {
            Node::Inner(inner) => {
                // Traverse through the left subtree, then the right subtree.
                if before_end {
                    self.delayed_nodes.push(inner.get_right_node(self.node_db));
                }

                if after_start {
                    self.delayed_nodes.push(inner.get_left_node(self.node_db));
                }
            }
            Node::Leaf(leaf) => {
                if self.range.contains(&leaf.key) {
                    // we have a leaf node within the range
                    return Some((leaf.key.clone(), leaf.value.clone()));
                }
            }
        }

        self.traverse()
    }
}

impl<'a, T: RangeBounds<Vec<u8>>, R: DB> Iterator for Range<'a, T, R> {
    type Item = (Vec<u8>, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        self.traverse()
    }
}

fn encode_bytes(bz: &[u8]) -> Vec<u8> {
    let mut enc_bytes = bz.len().encode_var_vec();
    enc_bytes.extend_from_slice(bz);

    return enc_bytes;
}

fn decode_bytes(bz: &[u8]) -> Result<(Vec<u8>, usize), Error> {
    let (bz_length, n_consumed) = usize::decode_var(&bz).ok_or(Error::NodeDeserialize)?;
    let bytes = bz[n_consumed..n_consumed + bz_length].to_vec();

    return Ok((bytes, n_consumed + bz_length));
}

#[cfg(test)]
mod tests {
    use super::*;
    use database::MemDB;

    #[test]
    fn right_rotate_works() {
        let t3 = InnerNode {
            left_node: Some(Arc::new(Node::Leaf(LeafNode {
                key: vec![19],
                value: vec![3, 2, 1],
                version: 0,
            }))),
            right_node: Some(Arc::new(Node::Leaf(LeafNode {
                key: vec![20],
                value: vec![1, 6, 9],
                version: 0,
            }))),
            key: vec![20],
            height: 1,
            size: 2,
            left_hash: [
                56, 18, 97, 18, 6, 216, 38, 113, 24, 103, 129, 119, 92, 30, 188, 114, 183, 100,
                110, 73, 39, 131, 243, 199, 251, 72, 125, 220, 56, 132, 125, 106,
            ],
            right_hash: [
                150, 105, 234, 135, 99, 29, 12, 162, 67, 236, 81, 117, 3, 18, 217, 76, 202, 161,
                168, 94, 102, 108, 58, 135, 122, 167, 228, 134, 150, 121, 201, 234,
            ],
            version: 0,
        };

        let y = InnerNode {
            left_node: Some(Arc::new(Node::Leaf(LeafNode {
                key: vec![18],
                value: vec![3, 2, 1],
                version: 0,
            }))),
            right_node: Some(Arc::new(Node::Inner(t3))),
            key: vec![19],
            height: 2,
            size: 3,
            left_hash: [
                93, 129, 120, 78, 65, 12, 13, 69, 115, 187, 137, 249, 49, 28, 235, 190, 117, 117,
                64, 156, 133, 127, 116, 73, 127, 31, 220, 155, 141, 243, 58, 254,
            ],
            right_hash: [
                192, 103, 168, 209, 21, 23, 137, 121, 173, 138, 179, 199, 124, 163, 200, 22, 101,
                85, 103, 102, 253, 118, 15, 195, 248, 223, 181, 228, 63, 234, 156, 135,
            ],
            version: 0,
        };

        let z = InnerNode {
            left_node: Some(Arc::new(Node::Inner(y))),
            right_node: Some(Arc::new(Node::Leaf(LeafNode {
                key: vec![21],
                value: vec![3, 2, 1],
                version: 0,
            }))),
            key: vec![21],
            height: 3,
            size: 4,
            left_hash: [
                99, 11, 87, 15, 142, 124, 184, 114, 169, 142, 60, 89, 127, 225, 44, 148, 55, 15,
                134, 99, 95, 20, 72, 212, 28, 163, 207, 203, 187, 144, 112, 183,
            ],
            right_hash: [
                0, 85, 79, 1, 62, 128, 35, 121, 122, 250, 9, 14, 106, 197, 49, 81, 58, 121, 9, 157,
                156, 44, 10, 204, 48, 235, 172, 20, 43, 158, 240, 254,
            ],
            version: 0,
        };

        let mut z = Node::Inner(z);

        let db = MemDB::new();
        Tree::right_rotate(&mut z, 0, &NodeDB::new(db)).unwrap();

        let hash = z.hash();
        let expected = [
            69, 219, 80, 128, 205, 82, 236, 60, 148, 147, 20, 32, 93, 192, 39, 130, 142, 68, 139,
            82, 137, 143, 154, 101, 208, 126, 98, 136, 17, 60, 138, 232,
        ];
        assert_eq!(hash, expected)
    }

    #[test]
    fn left_rotate_works() {
        let t2 = InnerNode {
            left_node: Some(Arc::new(Node::Leaf(LeafNode {
                key: vec![19],
                value: vec![3, 2, 1],
                version: 0,
            }))),
            right_node: Some(Arc::new(Node::Leaf(LeafNode {
                key: vec![20],
                value: vec![1, 6, 9],
                version: 0,
            }))),
            key: vec![20],
            height: 1,
            size: 2,
            left_hash: [
                56, 18, 97, 18, 6, 216, 38, 113, 24, 103, 129, 119, 92, 30, 188, 114, 183, 100,
                110, 73, 39, 131, 243, 199, 251, 72, 125, 220, 56, 132, 125, 106,
            ],
            right_hash: [
                150, 105, 234, 135, 99, 29, 12, 162, 67, 236, 81, 117, 3, 18, 217, 76, 202, 161,
                168, 94, 102, 108, 58, 135, 122, 167, 228, 134, 150, 121, 201, 234,
            ],
            version: 0,
        };

        let y = InnerNode {
            right_node: Some(Arc::new(Node::Leaf(LeafNode {
                key: vec![21],
                value: vec![3, 2, 1, 1],
                version: 0,
            }))),
            left_node: Some(Arc::new(Node::Inner(t2))),
            key: vec![21],
            height: 2,
            size: 3,
            right_hash: [
                228, 95, 46, 250, 156, 226, 109, 111, 149, 171, 184, 71, 170, 219, 77, 170, 113,
                216, 178, 65, 111, 142, 17, 195, 169, 129, 164, 6, 25, 91, 141, 173,
            ],
            left_hash: [
                192, 103, 168, 209, 21, 23, 137, 121, 173, 138, 179, 199, 124, 163, 200, 22, 101,
                85, 103, 102, 253, 118, 15, 195, 248, 223, 181, 228, 63, 234, 156, 135,
            ],
            version: 0,
        };

        let z = InnerNode {
            right_node: Some(Arc::new(Node::Inner(y))),
            left_node: Some(Arc::new(Node::Leaf(LeafNode {
                key: vec![18],
                value: vec![3, 2, 2],
                version: 0,
            }))),
            key: vec![19],
            height: 3,
            size: 4,
            left_hash: [
                121, 226, 107, 73, 123, 135, 165, 82, 94, 53, 112, 50, 126, 200, 252, 137, 235, 87,
                205, 133, 96, 202, 94, 222, 39, 138, 231, 198, 189, 196, 49, 196,
            ],
            right_hash: [
                13, 181, 53, 227, 140, 38, 242, 22, 94, 152, 94, 71, 0, 89, 35, 122, 129, 85, 55,
                190, 253, 226, 35, 230, 65, 214, 244, 35, 69, 39, 223, 90,
            ],
            version: 0,
        };

        let mut z = Node::Inner(z);

        let db = MemDB::new();
        Tree::left_rotate(&mut z, 0, &NodeDB::new(db)).unwrap();

        let hash = z.hash();
        let expected = [
            221, 58, 23, 0, 25, 206, 49, 41, 174, 43, 173, 118, 31, 30, 46, 172, 195, 159, 69, 125,
            238, 68, 72, 17, 217, 148, 126, 112, 95, 17, 115, 160,
        ];
        assert_eq!(hash, expected)
    }

    #[test]
    fn set_equal_leaf_works() {
        let db = MemDB::new();
        let mut tree = Tree::new(db, None).unwrap();
        tree.set(vec![1], vec![2]);
        tree.set(vec![1], vec![3]);

        let hash = tree.root_hash();
        let expected = [
            146, 114, 60, 233, 157, 240, 49, 35, 57, 65, 154, 83, 84, 160, 123, 45, 153, 137, 215,
            139, 195, 141, 74, 219, 86, 182, 75, 239, 223, 87, 133, 81,
        ];
        assert_eq!(hash, expected)
    }

    #[test]
    fn set_less_than_leaf_works() {
        let db = MemDB::new();
        let mut tree = Tree::new(db, None).unwrap();
        tree.set(vec![3], vec![2]);
        tree.set(vec![1], vec![3]);

        let hash = tree.root_hash();
        let expected = [
            197, 117, 162, 213, 61, 146, 253, 165, 111, 237, 42, 95, 186, 76, 202, 167, 174, 187,
            19, 6, 150, 29, 243, 41, 209, 142, 80, 45, 32, 9, 235, 24,
        ];
        assert_eq!(hash, expected)
    }

    #[test]
    fn set_greater_than_leaf_works() {
        let db = MemDB::new();
        let mut tree = Tree::new(db, None).unwrap();
        tree.set(vec![1], vec![2]);
        tree.set(vec![3], vec![3]);

        let hash = tree.root_hash();
        let expected = [
            27, 213, 240, 14, 167, 98, 231, 104, 130, 46, 40, 228, 172, 2, 149, 149, 32, 10, 198,
            129, 179, 18, 29, 182, 227, 231, 178, 29, 160, 69, 142, 244,
        ];
        assert_eq!(hash, expected)
    }

    #[test]
    fn repeated_set_works() {
        let db = MemDB::new();
        let mut tree = Tree::new(db, None).unwrap();
        tree.set(b"alice".to_vec(), b"abc".to_vec());
        tree.set(b"bob".to_vec(), b"123".to_vec());
        tree.set(b"c".to_vec(), b"1".to_vec());
        tree.set(b"q".to_vec(), b"1".to_vec());

        let expected = [
            202, 52, 159, 10, 210, 166, 72, 207, 248, 190, 60, 114, 172, 147, 84, 27, 120, 202,
            189, 127, 230, 108, 58, 127, 251, 149, 9, 33, 87, 249, 158, 138,
        ];

        assert_eq!(expected, tree.root_hash());
    }

    #[test]
    fn save_version_works() {
        let db = MemDB::new();
        let mut tree = Tree::new(db, None).unwrap();
        tree.set(b"alice".to_vec(), b"abc".to_vec());
        tree.set(b"bob".to_vec(), b"123".to_vec());
        tree.set(b"c".to_vec(), b"1".to_vec());
        tree.set(b"q".to_vec(), b"1".to_vec());

        tree.save_version();
        tree.save_version();
        tree.set(b"qwerty".to_vec(), b"312".to_vec());
        tree.set(b"-32".to_vec(), b"gamma".to_vec());
        tree.save_version();
        tree.set(b"alice".to_vec(), b"123".to_vec());
        tree.save_version();

        let expected = [
            37, 155, 233, 229, 243, 173, 29, 241, 235, 234, 85, 10, 36, 129, 53, 79, 77, 11, 29,
            118, 201, 233, 133, 60, 78, 187, 37, 81, 42, 96, 105, 150,
        ];

        assert_eq!(expected, tree.root_hash());
    }

    #[test]
    fn get_works() {
        let db = MemDB::new();
        let mut tree = Tree::new(db, None).unwrap();
        tree.set(b"alice".to_vec(), b"abc".to_vec());
        tree.set(b"bob".to_vec(), b"123".to_vec());
        tree.set(b"c".to_vec(), b"1".to_vec());
        tree.set(b"q".to_vec(), b"1".to_vec());

        assert_eq!(tree.get(b"alice"), Some(String::from("abc").into()));
        assert_eq!(tree.get(b"bob"), Some(String::from("123").into()));
        assert_eq!(tree.get(b"c"), Some(String::from("1").into()));
        assert_eq!(tree.get(b"q"), Some(String::from("1").into()));
        assert_eq!(tree.get(b"house"), None);
    }

    #[test]
    fn scenario_works() {
        let db = MemDB::new();
        let mut tree = Tree::new(db, None).unwrap();
        tree.set(vec![0, 117, 97, 116, 111, 109], vec![51, 52]);
        tree.set(
            vec![
                2, 20, 129, 58, 194, 42, 97, 73, 22, 85, 226, 120, 106, 224, 209, 39, 214, 153, 11,
                251, 251, 222, 117, 97, 116, 111, 109,
            ],
            vec![10, 5, 117, 97, 116, 111, 109, 18, 2, 51, 52],
        );

        tree.save_version();
        tree.save_version();
        tree.save_version();
        tree.save_version();
        tree.save_version();
        tree.save_version();
        tree.save_version();

        tree.set(
            vec![
                2, 20, 59, 214, 51, 187, 112, 177, 248, 133, 197, 68, 36, 228, 124, 164, 14, 68,
                72, 143, 236, 46, 117, 97, 116, 111, 109,
            ],
            vec![10, 5, 117, 97, 116, 111, 109, 18, 2, 49, 48],
        );
        tree.set(
            vec![
                2, 20, 129, 58, 194, 42, 97, 73, 22, 85, 226, 120, 106, 224, 209, 39, 214, 153, 11,
                251, 251, 222, 117, 97, 116, 111, 109,
            ],
            vec![10, 5, 117, 97, 116, 111, 109, 18, 2, 50, 51],
        );
        tree.set(
            vec![
                2, 20, 241, 130, 150, 118, 219, 87, 118, 130, 233, 68, 252, 52, 147, 212, 81, 182,
                127, 243, 226, 159, 117, 97, 116, 111, 109,
            ],
            vec![10, 5, 117, 97, 116, 111, 109, 18, 1, 49],
        );

        let expected = [
            34, 215, 64, 141, 118, 237, 192, 198, 47, 22, 34, 81, 0, 146, 145, 66, 182, 59, 101,
            145, 99, 187, 82, 49, 149, 36, 196, 63, 37, 42, 171, 124,
        ];

        let (hash, version) = tree.save_version();

        assert_eq!((expected, 8), (hash, version));
    }

    #[test]
    fn bounded_range_works() {
        let db = MemDB::new();
        let mut tree = Tree::new(db, None).unwrap();
        tree.set(b"1".to_vec(), b"abc1".to_vec());

        tree.set(b"2".to_vec(), b"abc2".to_vec());
        tree.set(b"3".to_vec(), b"abc3".to_vec());
        tree.set(b"4".to_vec(), b"abc4".to_vec());
        tree.set(b"5".to_vec(), b"abc5".to_vec());
        tree.set(b"6".to_vec(), b"abc6".to_vec());
        tree.set(b"7".to_vec(), b"abc7".to_vec());

        // [,)
        let start = b"3".to_vec();
        let stop = b"6".to_vec();
        let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = tree.range(start..stop).collect();
        let expected_pairs = vec![
            (b"3".to_vec(), b"abc3".to_vec()),
            (b"4".to_vec(), b"abc4".to_vec()),
            (b"5".to_vec(), b"abc5".to_vec()),
        ];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.into_iter().all(|e| {
            let cmp = (e.0, e.1);
            got_pairs.contains(&cmp)
        }));

        // [,]
        let start = b"3".to_vec();
        let stop = b"6".to_vec();
        let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = tree.range(start..=stop).collect();
        let expected_pairs = vec![
            (b"3".to_vec(), b"abc3".to_vec()),
            (b"4".to_vec(), b"abc4".to_vec()),
            (b"5".to_vec(), b"abc5".to_vec()),
            (b"6".to_vec(), b"abc6".to_vec()),
        ];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.into_iter().all(|e| {
            let cmp = (e.0, e.1);
            got_pairs.contains(&cmp)
        }));

        // (,)
        let start = b"3".to_vec();
        let stop = b"6".to_vec();
        let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = tree
            .range((Bound::Excluded(start), Bound::Excluded(stop)))
            .collect();
        let expected_pairs = vec![
            (b"4".to_vec(), b"abc4".to_vec()),
            (b"5".to_vec(), b"abc5".to_vec()),
        ];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.into_iter().all(|e| {
            let cmp = (e.0, e.1);
            got_pairs.contains(&cmp)
        }));
    }

    #[test]
    fn full_range_unique_keys_works() {
        let db = MemDB::new();
        let mut tree = Tree::new(db, None).unwrap();
        tree.set(b"alice".to_vec(), b"abc".to_vec());
        tree.set(b"bob".to_vec(), b"123".to_vec());
        tree.set(b"c".to_vec(), b"1".to_vec());
        tree.set(b"q".to_vec(), b"1".to_vec());
        let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = tree.range(..).collect();

        let expected_pairs = vec![
            (b"alice".to_vec(), b"abc".to_vec()),
            (b"c".to_vec(), b"1".to_vec()),
            (b"q".to_vec(), b"1".to_vec()),
            (b"bob".to_vec(), b"123".to_vec()),
        ];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.into_iter().all(|e| {
            let cmp = (e.0, e.1);
            got_pairs.contains(&cmp)
        }));
    }

    #[test]
    fn full_range_duplicate_keys_works() {
        let db = MemDB::new();
        let mut tree = Tree::new(db, None).unwrap();
        tree.set(b"alice".to_vec(), b"abc".to_vec());
        tree.set(b"alice".to_vec(), b"abc".to_vec());
        let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = tree.range(..).collect();

        let expected_pairs = vec![(b"alice".to_vec(), b"abc".to_vec())];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.into_iter().all(|e| {
            let cmp = (e.0, e.1);
            got_pairs.contains(&cmp)
        }));
    }

    #[test]
    fn empty_tree_range_works() {
        let db = MemDB::new();
        let mut tree = Tree::new(db, None).unwrap();
        let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = tree.range(..).collect();

        let expected_pairs: Vec<(Vec<u8>, Vec<u8>)> = vec![];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.into_iter().all(|e| {
            let cmp = (e.0, e.1);
            got_pairs.contains(&cmp)
        }));
    }
}
