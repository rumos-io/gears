use database::DB;
use integer_encoding::VarInt;

#[derive(Debug)]
pub struct NodeDB<T>
where
    T: DB,
{
    db: T,
}

const ROOTS_PREFIX: [u8; 1] = [1];

impl<T> NodeDB<T>
where
    T: DB,
{
    pub fn new(db: T) -> NodeDB<T> {
        NodeDB { db }
    }

    pub fn get_roots(&self) -> Vec<(u32, Box<[u8]>)> {
        self.db
            .prefix_iterator(ROOTS_PREFIX.into())
            .map(|(k, v)| {
                let version = u32::decode_var(&k)
                    .expect("expect this to be a valid u32")
                    .0;
                (version, v)
            })
            .collect()
    }
}
