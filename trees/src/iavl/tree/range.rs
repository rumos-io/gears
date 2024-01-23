use std::ops::{Bound, RangeBounds};

use super::node::Node;

pub struct Range<R: RangeBounds<Vec<u8>>> {
    pub(crate) range: R,
    pub(crate) delayed_nodes: Vec<Node>,
}

impl<T: RangeBounds<Vec<u8>>> Range<T> {
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
            if let Some(right_node) = &node.left_node {
                self.delayed_nodes.push((**right_node).clone()); //TODO: deref will cause a clone, remove
            }
        }

        if after_start {
            if let Some(left_node) = &node.left_node {
                self.delayed_nodes.push((**left_node).clone()); //TODO: deref will cause a clone, remove
            }
        }

        self.traverse()
    }
}

impl<T: RangeBounds<Vec<u8>>> Iterator for Range<T> {
    type Item = (Vec<u8>, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        self.traverse()
    }
}
