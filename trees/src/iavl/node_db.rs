use std::sync::Arc;

use database::{PrefixDB, RocksDB, DB};
use integer_encoding::VarInt;

#[derive(Debug)]
pub struct NodeDB<T>
where
    T: DB,
{
    db: T,
}

impl<T> NodeDB<T>
where
    T: DB,
{
    pub fn new(db: T) -> NodeDB<T> {
        NodeDB { db }
    }

    pub fn get_roots(&self) -> Vec<(u32, Box<[u8]>)> {
        //TODO: the roots need to be prefixed
        self.db
            .iterator()
            .map(|(k, v)| {
                let version = u32::decode_var(&k)
                    .expect("expect this to be a valid u32")
                    .0;
                (version, v)
            })
            .collect()
    }
}
