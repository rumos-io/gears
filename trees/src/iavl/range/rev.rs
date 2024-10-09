use std::{
    marker::PhantomData,
    ops::{Bound, RangeBounds},
};

use database::Database;

use crate::iavl::{node_db::NodeDB, Node};

#[derive(Debug, Clone)]
pub struct RevRange<'a, DB, RB, R> {
    pub(super) range: R,
    pub(super) delayed_nodes: Vec<Box<Node>>,
    pub(super) node_db: &'a NodeDB<DB>,
    pub(super) _marker: PhantomData<RB>,
}

impl<DB: Database, R: RangeBounds<RB>, RB: AsRef<[u8]>> RevRange<'_, DB, RB, R> {
    fn traverse(&mut self) -> Option<(Vec<u8>, Vec<u8>)> {
        // Instead usage of VecDeque I assume Vec already rev in `new`(or similar)
        let node = self.delayed_nodes.pop()?;

        let after_start = match self.range.start_bound() {
            Bound::Included(l) => node.get_key() > l.as_ref(),
            Bound::Excluded(l) => node.get_key() > l.as_ref(),
            Bound::Unbounded => true,
        };

        let before_end = match self.range.end_bound() {
            Bound::Included(u) => node.get_key() <= u.as_ref(),
            Bound::Excluded(u) => node.get_key() < u.as_ref(),
            Bound::Unbounded => true,
        };

        match *node {
            Node::Leaf(leaf) => {
                // TODO: Replace with (after_start && before_end) when add tests
                let is_contains = (match self.range.start_bound() {
                    Bound::Included(start) => start.as_ref() <= &leaf.key,
                    Bound::Excluded(start) => start.as_ref() < &leaf.key,
                    Bound::Unbounded => true,
                }) && (match self.range.end_bound() {
                    Bound::Included(end) => leaf.key.as_slice() <= end.as_ref(),
                    Bound::Excluded(end) => leaf.key.as_slice() < end.as_ref(),
                    Bound::Unbounded => true,
                });

                if is_contains {
                    // we have a leaf node within the range
                    return Some((leaf.key, leaf.value));
                }
            }
            Node::Inner(inner) => {
                // Traverse through the right subtree, then the left subtree.
                if before_end {
                    match inner.left_node {
                        Some(left_node) => self.delayed_nodes.push(left_node),
                        None => {
                            let left_node = self
                                .node_db
                                .get_node(&inner.left_hash)
                                .expect("node db should contain all nodes");

                            self.delayed_nodes.push(left_node);
                        }
                    }
                }

                if after_start {
                    match inner.right_node {
                        Some(right_node) => self.delayed_nodes.push(right_node),
                        None => {
                            let right_node = self
                                .node_db
                                .get_node(&inner.right_hash)
                                .expect("node db should contain all nodes");

                            self.delayed_nodes.push(right_node);
                        }
                    }
                }
            }
        }

        self.traverse()
    }
}

impl<DB: Database, R: RangeBounds<RB>, RB: AsRef<[u8]>> Iterator for RevRange<'_, DB, RB, R> {
    type Item = (Vec<u8>, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        self.traverse()
    }
}


