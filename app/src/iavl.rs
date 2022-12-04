use std::cmp;

use crate::error::IAVLError;

pub enum Node {
    Inner(InnerNode),
    Leaf(LeafNode),
}

pub struct InnerNode {
    left_node: Box<Node>,
    right_node: Box<Node>,
    key: u64,
    height: u8,
}

pub struct LeafNode {
    key: u64,
    value: u64,
}

pub struct IAVLTree {
    root: Node,
}

impl IAVLTree {
    pub fn set(tree: IAVLTree, key: u64, value: u64) -> Self {
        IAVLTree {
            root: Self::recursive_set(tree.root, key, value),
        }
    }

    fn recursive_set(node: Node, key: u64, value: u64) -> Node {
        match node {
            Node::Leaf(mut node) => {
                match key.cmp(&node.key) {
                    cmp::Ordering::Less => {
                        let left_node = LeafNode { key, value };
                        let root = InnerNode {
                            key: node.key,
                            left_node: Box::new(Node::Leaf(left_node)),
                            right_node: Box::new(Node::Leaf(node)),
                            height: 1,
                        };
                        return Node::Inner(root);
                    }
                    cmp::Ordering::Equal => {
                        node.value = value;
                        return Node::Leaf(node);
                    }
                    cmp::Ordering::Greater => {
                        let right_node = LeafNode { key, value };
                        let root = InnerNode {
                            key: node.key,
                            left_node: Box::new(Node::Leaf(node)),
                            right_node: Box::new(Node::Leaf(right_node)),
                            height: 1,
                        };
                        return Node::Inner(root);
                    }
                };
            }
            Node::Inner(mut node) => {
                // Perform normal BST
                if key < node.key {
                    node.left_node = Box::new(Self::recursive_set(*node.left_node, key, value));
                } else {
                    node.right_node = Box::new(Self::recursive_set(*node.right_node, key, value));
                }

                // Update height
                node.height = 1 + cmp::max(
                    Self::get_height(&node.left_node),
                    Self::get_height(&node.right_node),
                );

                // If the node is unbalanced then try out the usual four cases
                let balance_factor = Self::get_balance_factor(&node);
                if balance_factor > 1 {
                    match *node.left_node {
                        Node::Leaf(_) => {
                            panic!("Given balance factor > 1, expect this to be an inner node")
                        }
                        Node::Inner(left_node) => {
                            // Case 1 - Left Left
                            if key < left_node.key {
                                // move the left node back!
                                node.left_node = Box::new(Node::Inner(left_node));
                                return Node::Inner(
                                    Self::right_rotate(node)
                                        .expect("Expect rotation to always succeed"),
                                );
                            // Case 2 - Left Right
                            } else {
                                node.left_node = Box::new(Node::Inner(
                                    Self::left_rotate(left_node)
                                        .expect("Expect rotation to always succeed"),
                                ));
                                return Node::Inner(
                                    Self::right_rotate(node)
                                        .expect("Expect rotation to always succeed"),
                                );
                            }
                        }
                    }
                } else if balance_factor < -1 {
                    match *node.right_node {
                        Node::Leaf(_) => {
                            panic!("Given balance factor < -1, expect this to be an inner node")
                        }
                        Node::Inner(right_node) => {
                            // Case 3 - Right Right
                            if key > right_node.key {
                                // move the right node back!
                                node.right_node = Box::new(Node::Inner(right_node));
                                return Node::Inner(
                                    Self::left_rotate(node)
                                        .expect("Expect rotation to always succeed"),
                                );
                            //Case 4 - Right Left
                            } else {
                                node.right_node = Box::new(Node::Inner(
                                    Self::right_rotate(right_node)
                                        .expect("Expect rotation to always succeed"),
                                ));
                                return Node::Inner(
                                    Self::left_rotate(node)
                                        .expect("Expect rotation to always succeed"),
                                );
                            }
                        }
                    }
                }

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

    fn get_balance_factor(node: &InnerNode) -> i16 {
        let left_height: i16 = Self::get_height(&node.left_node).into();
        let right_height: i16 = Self::get_height(&node.right_node).into();
        left_height - right_height
    }

    fn right_rotate(mut z: InnerNode) -> Result<InnerNode, IAVLError> {
        let y = z.left_node;

        let mut y = match *y {
            Node::Inner(y) => y,
            Node::Leaf(_) => return Err(IAVLError::RotateError),
        };

        let t3 = y.right_node;

        // Perform rotation on z and update height
        z.left_node = t3;
        z.height = 1 + cmp::max(
            Self::get_height(&z.left_node),
            Self::get_height(&z.right_node),
        );

        // Perform rotation on y and update height
        y.right_node = Box::new(Node::Inner(z));
        y.height = 1 + cmp::max(
            Self::get_height(&y.left_node),
            Self::get_height(&y.right_node),
        );

        // Return the new root
        return Ok(y);
    }

    fn left_rotate(mut z: InnerNode) -> Result<InnerNode, IAVLError> {
        let y = z.right_node;

        let mut y = match *y {
            Node::Inner(y) => y,
            Node::Leaf(_) => return Err(IAVLError::RotateError),
        };

        let t2 = y.left_node;

        // Perform rotation on z and update height
        z.right_node = t2;
        z.height = 1 + cmp::max(
            Self::get_height(&z.left_node),
            Self::get_height(&z.right_node),
        );

        // Perform rotation on y and update height
        y.left_node = Box::new(Node::Inner(z));
        y.height = 1 + cmp::max(
            Self::get_height(&y.left_node),
            Self::get_height(&y.right_node),
        );

        // Return the new root
        return Ok(y);
    }
}

// #[cfg(test)]
// mod tests {

//     use super::*;

//     fn make_node(height: u8, key: u64, value: u64) -> Node {
//         Node {
//             left_node: None,
//             right_node: None,
//             height,
//             key,
//             value,
//         }
//     }

//     fn make_tree() {}

//     #[test]
//     fn get_height_works() {
//         let height = IAVLTree::get_height(&None);
//         assert_eq!(height, 0);

//         let height = IAVLTree::get_height(&Some(Box::new(Node {
//             left_node: None,
//             right_node: None,
//             height: 100,
//             key: 0,
//             value: 0,
//         })));
//         assert_eq!(height, 100);
//     }

//     #[test]
//     fn get_balance_factor_works() {}

//     // #[test]
//     // fn right_rotate() {
//     //     let node = Node {
//     //         left_node: None,
//     //         right_node: todo!(),
//     //         height: todo!(),
//     //         key: todo!(),
//     //         value: todo!(),
//     //     };

//     //     // IAVLTree::right_rotate(x, y)
//     //     // let expected = Uint256::from(300u128);
//     //     // assert_eq!(Decimal256::new(expected).0, expected);
//     // }
// }
