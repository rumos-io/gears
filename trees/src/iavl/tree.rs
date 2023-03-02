use std::{
    cmp,
    ops::{Bound, RangeBounds},
};

use crate::{error::Error, merkle::EMPTY_HASH};

use super::node::{InnerNode, LeafNode, Node};

#[derive(Debug, Clone)]
pub struct Tree {
    root: Option<Node>,
    version: u32,
}

impl Tree {
    pub fn new() -> Tree {
        Tree {
            root: None,
            version: 0,
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

    pub fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        match &self.root {
            Some(root) => Tree::recursive_get(root, key),
            None => None,
        }
    }

    fn recursive_get<'a>(node: &'a Node, key: &[u8]) -> Option<&'a Vec<u8>> {
        match node {
            Node::Leaf(leaf) => {
                if leaf.key == key {
                    return Some(&leaf.value);
                } else {
                    return None;
                }
            }
            Node::Inner(node) => {
                if key < &node.key {
                    return Tree::recursive_get(&node.left_node, key);
                } else {
                    return Tree::recursive_get(&node.right_node, key);
                }
            }
        }
    }

    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.root = match &self.root {
            Some(root) => {
                // TODO: recursive_set should take a mutable reference to avoid cloning the node here
                Some(Self::recursive_set(
                    root.clone(),
                    key,
                    value,
                    self.version + 1,
                ))
            }
            None => Some(Node::Leaf(LeafNode {
                key: key,
                value: value,
                version: 1, //TODO: should this be self.version + 1
            })),
        };
    }

    fn recursive_set(node: Node, key: Vec<u8>, value: Vec<u8>, version: u32) -> Node {
        match node {
            Node::Leaf(mut node) => {
                match key.cmp(&node.key) {
                    cmp::Ordering::Less => {
                        let left_node = Node::Leaf(LeafNode {
                            key,
                            value,
                            version,
                        });

                        let left_hash = left_node.hash();

                        let key = node.key.clone();
                        let right_node = Node::Leaf(node);
                        let right_hash = right_node.hash();

                        let root = InnerNode {
                            key: key.clone(),
                            left_node: Box::new(left_node),
                            right_node: Box::new(right_node),
                            height: 1,
                            size: 2,
                            version,
                            left_hash,
                            right_hash,
                        };
                        return Node::Inner(root);
                    }
                    cmp::Ordering::Equal => {
                        node.value = value;
                        node.version = version;
                        return Node::Leaf(node);
                    }
                    cmp::Ordering::Greater => {
                        let right_node = Node::Leaf(LeafNode {
                            key: key.clone(),
                            value,
                            version,
                        });

                        let right_hash = right_node.hash();

                        //let key = node.key;
                        let left_node = Node::Leaf(node);
                        let left_hash = left_node.hash();

                        let root = InnerNode {
                            key,
                            left_node: Box::new(left_node),
                            right_node: Box::new(right_node),
                            height: 1,
                            size: 2,
                            left_hash,
                            right_hash,
                            version,
                        };
                        return Node::Inner(root);
                    }
                };
            }
            Node::Inner(mut node) => {
                // Perform normal BST
                if key < node.key {
                    node.left_node = Box::new(Self::recursive_set(
                        *node.left_node,
                        key.clone(),
                        value,
                        version,
                    ));
                    node.left_hash = node.left_node.hash();
                } else {
                    node.right_node = Box::new(Self::recursive_set(
                        *node.right_node,
                        key.clone(),
                        value,
                        version,
                    ));
                    node.right_hash = node.right_node.hash();
                }

                // Update height
                node.height = 1 + cmp::max(
                    Self::get_height(&node.left_node),
                    Self::get_height(&node.right_node),
                );

                node.size = Self::get_size(&node.left_node) + Self::get_size(&node.right_node);

                // If the node is unbalanced then try out the usual four cases
                let balance_factor = Self::get_balance_factor(&node);
                if balance_factor > 1 {
                    match *node.left_node {
                        Node::Leaf(_) => {
                            panic!("Since balance factor > 1, expect this to be an inner node")
                        }
                        Node::Inner(left_node) => {
                            // Case 1 - Left Left
                            if key < left_node.key {
                                // move the left node back!
                                node.left_node = Box::new(Node::Inner(left_node));
                                return Node::Inner(
                                    Self::right_rotate(node, version)
                                        .expect("Expect rotation to always succeed"),
                                );
                            // Case 2 - Left Right
                            } else {
                                node.left_node = Box::new(Node::Inner(
                                    Self::left_rotate(left_node, version)
                                        .expect("Expect rotation to always succeed"),
                                ));
                                return Node::Inner(
                                    Self::right_rotate(node, version)
                                        .expect("Expect rotation to always succeed"),
                                );
                            }
                        }
                    }
                } else if balance_factor < -1 {
                    match *node.right_node {
                        Node::Leaf(_) => {
                            panic!("Since balance factor < -1, expect this to be an inner node")
                        }
                        Node::Inner(right_node) => {
                            // Case 3 - Right Right
                            if key > right_node.key {
                                // move the right node back!
                                node.right_node = Box::new(Node::Inner(right_node));
                                return Node::Inner(
                                    Self::left_rotate(node, version)
                                        .expect("Expect rotation to always succeed"),
                                );
                            //Case 4 - Right Left
                            } else {
                                node.right_node = Box::new(Node::Inner(
                                    Self::right_rotate(right_node, version)
                                        .expect("Expect rotation to always succeed"),
                                ));
                                return Node::Inner(
                                    Self::left_rotate(node, version)
                                        .expect("Expect rotation to always succeed"),
                                );
                            }
                        }
                    }
                }

                node.version = version;

                return Node::Inner(node);
            }
        };
    }

    fn get_height(node: &Node) -> u8 {
        match node {
            Node::Leaf(_) => 0,
            Node::Inner(n) => n.height,
        }
    }

    fn get_size(node: &Node) -> u32 {
        match node {
            Node::Leaf(_) => 1,
            Node::Inner(n) => n.size,
        }
    }

    fn get_balance_factor(node: &InnerNode) -> i16 {
        let left_height: i16 = Self::get_height(&node.left_node).into();
        let right_height: i16 = Self::get_height(&node.right_node).into();
        left_height - right_height
    }

    fn right_rotate(mut z: InnerNode, version: u32) -> Result<InnerNode, Error> {
        let y = z.left_node;

        let mut y = match *y {
            Node::Inner(y) => y,
            Node::Leaf(_) => return Err(Error::RotateError),
        };

        let t3 = y.right_node;

        // Perform rotation on z and update height and hash
        z.left_node = t3;
        z.height = 1 + cmp::max(
            Self::get_height(&z.left_node),
            Self::get_height(&z.right_node),
        );
        z.size = Self::get_size(&z.left_node) + Self::get_size(&z.right_node);
        z.version = version;
        z.left_hash = y.right_hash;
        let z = Node::Inner(z);

        // Perform rotation on y, update hash and update height
        y.right_hash = z.hash();
        y.right_node = Box::new(z);
        y.height = 1 + cmp::max(
            Self::get_height(&y.left_node),
            Self::get_height(&y.right_node),
        );
        y.size = Self::get_size(&y.left_node) + Self::get_size(&y.right_node);
        y.version = version;

        // Return the new root
        return Ok(y);
    }

    fn left_rotate(mut z: InnerNode, version: u32) -> Result<InnerNode, Error> {
        let y = z.right_node;

        let mut y = match *y {
            Node::Inner(y) => y,
            Node::Leaf(_) => return Err(Error::RotateError),
        };

        let t2 = y.left_node;

        // Perform rotation on z and update height and hash
        z.right_node = t2;
        z.height = 1 + cmp::max(
            Self::get_height(&z.left_node),
            Self::get_height(&z.right_node),
        );
        z.size = Self::get_size(&z.left_node) + Self::get_size(&z.right_node);
        z.version = version;
        z.right_hash = y.left_hash;
        let z = Node::Inner(z);

        // Perform rotation on y, update hash and update height
        y.left_hash = z.hash();
        y.left_node = Box::new(z);
        y.height = 1 + cmp::max(
            Self::get_height(&y.left_node),
            Self::get_height(&y.right_node),
        );
        y.size = Self::get_size(&y.left_node) + Self::get_size(&y.right_node);
        y.version = version;

        // Return the new root
        return Ok(y);
    }

    pub fn range<R>(&self, range: R) -> Range<R>
    where
        R: RangeBounds<Vec<u8>>,
    {
        match &self.root {
            Some(node) => Range {
                range,
                delayed_nodes: vec![&node],
            },
            None => Range {
                range,
                delayed_nodes: vec![],
            },
        }
    }
}

pub struct Range<'a, R: RangeBounds<Vec<u8>>> {
    range: R,
    delayed_nodes: Vec<&'a Node>,
}

impl<'a, T: RangeBounds<Vec<u8>>> Range<'a, T> {
    fn traverse(&mut self) -> Option<(&'a Vec<u8>, &'a Vec<u8>)> {
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

        match node {
            Node::Inner(node) => {
                // Traverse through the left subtree, then the right subtree.
                if before_end {
                    self.delayed_nodes.push(&node.right_node);
                }

                if after_start {
                    self.delayed_nodes.push(&node.left_node);
                }
            }
            Node::Leaf(node) => {
                if self.range.contains(&node.key) {
                    // we have a leaf node within the range
                    return Some((&node.key, &node.value));
                }
            }
        }

        self.traverse()
    }
}

impl<'a, T: RangeBounds<Vec<u8>>> Iterator for Range<'a, T> {
    type Item = (&'a Vec<u8>, &'a Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        self.traverse()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn repeated_set_works() {
        let mut tree = Tree::new();
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
        let mut tree = Tree::new();
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
        let mut tree = Tree::new();
        tree.set(b"alice".to_vec(), b"abc".to_vec());
        tree.set(b"bob".to_vec(), b"123".to_vec());
        tree.set(b"c".to_vec(), b"1".to_vec());
        tree.set(b"q".to_vec(), b"1".to_vec());

        assert_eq!(tree.get(b"alice"), Some(&String::from("abc").into()));
        assert_eq!(tree.get(b"bob"), Some(&String::from("123").into()));
        assert_eq!(tree.get(b"c"), Some(&String::from("1").into()));
        assert_eq!(tree.get(b"q"), Some(&String::from("1").into()));
        assert_eq!(tree.get(b"house"), None);
    }

    #[test]
    fn scenario_works() {
        let mut tree = Tree::new();
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
        let mut tree = Tree::new();
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
        let got_pairs: Vec<(&Vec<u8>, &Vec<u8>)> = tree.range(start..stop).collect();
        let expected_pairs = vec![
            (b"3".to_vec(), b"abc3".to_vec()),
            (b"4".to_vec(), b"abc4".to_vec()),
            (b"5".to_vec(), b"abc5".to_vec()),
        ];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.iter().all(|e| {
            let cmp = (&e.0, &e.1);
            got_pairs.contains(&cmp)
        }));

        // [,]
        let start = b"3".to_vec();
        let stop = b"6".to_vec();
        let got_pairs: Vec<(&Vec<u8>, &Vec<u8>)> = tree.range(start..=stop).collect();
        let expected_pairs = vec![
            (b"3".to_vec(), b"abc3".to_vec()),
            (b"4".to_vec(), b"abc4".to_vec()),
            (b"5".to_vec(), b"abc5".to_vec()),
            (b"6".to_vec(), b"abc6".to_vec()),
        ];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.iter().all(|e| {
            let cmp = (&e.0, &e.1);
            got_pairs.contains(&cmp)
        }));

        // (,)
        let start = b"3".to_vec();
        let stop = b"6".to_vec();
        let got_pairs: Vec<(&Vec<u8>, &Vec<u8>)> = tree
            .range((Bound::Excluded(start), Bound::Excluded(stop)))
            .collect();
        let expected_pairs = vec![
            (b"4".to_vec(), b"abc4".to_vec()),
            (b"5".to_vec(), b"abc5".to_vec()),
        ];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.iter().all(|e| {
            let cmp = (&e.0, &e.1);
            got_pairs.contains(&cmp)
        }));
    }

    #[test]
    fn full_range_unique_keys_works() {
        let mut tree = Tree::new();
        tree.set(b"alice".to_vec(), b"abc".to_vec());
        tree.set(b"bob".to_vec(), b"123".to_vec());
        tree.set(b"c".to_vec(), b"1".to_vec());
        tree.set(b"q".to_vec(), b"1".to_vec());
        let got_pairs: Vec<(&Vec<u8>, &Vec<u8>)> = tree.range(..).collect();

        let expected_pairs = vec![
            (b"alice".to_vec(), b"abc".to_vec()),
            (b"c".to_vec(), b"1".to_vec()),
            (b"q".to_vec(), b"1".to_vec()),
            (b"bob".to_vec(), b"123".to_vec()),
        ];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.iter().all(|e| {
            let cmp = (&e.0, &e.1);
            got_pairs.contains(&cmp)
        }));
    }

    #[test]
    fn full_range_duplicate_keys_works() {
        let mut tree = Tree::new();
        tree.set(b"alice".to_vec(), b"abc".to_vec());
        tree.set(b"alice".to_vec(), b"abc".to_vec());
        let got_pairs: Vec<(&Vec<u8>, &Vec<u8>)> = tree.range(..).collect();

        let expected_pairs = vec![(b"alice".to_vec(), b"abc".to_vec())];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.iter().all(|e| {
            let cmp = (&e.0, &e.1);
            got_pairs.contains(&cmp)
        }));
    }
}
