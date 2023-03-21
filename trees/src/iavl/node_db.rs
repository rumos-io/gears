use std::collections::BTreeSet;

use database::DB;
use integer_encoding::VarInt;

use super::Node;

#[derive(Debug)]
pub struct NodeDB<T>
where
    T: DB,
{
    db: T,
}

const ROOTS_PREFIX: [u8; 1] = [1];
const NODES_PREFIX: [u8; 1] = [2];

impl<T> NodeDB<T>
where
    T: DB,
{
    pub fn new(db: T) -> NodeDB<T> {
        NodeDB { db }
    }

    pub fn get_versions(&self) -> BTreeSet<u32> {
        self.db
            .prefix_iterator(ROOTS_PREFIX.into())
            .map(|(k, _)| {
                u32::decode_var(&k)
                    .expect("expect this to be a valid u32")
                    .0
            })
            .collect()
    }

    pub(crate) fn get_root(&self, version: u32) -> Option<Node> {
        let root_hash = self
            .db
            .get(&[ROOTS_PREFIX.into(), version.encode_var_vec()].concat())?;
        let node_bytes = self
            .db
            .get(&[NODES_PREFIX.into(), root_hash].concat())
            .expect("root node should be in DB");

        Some(
            Node::deserialize(node_bytes)
                .expect("invalid data in database - possible database corruption"),
        )
    }
}
