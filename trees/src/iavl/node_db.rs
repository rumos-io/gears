use std::{collections::BTreeSet, sync::Mutex};

use caches::{Cache, DefaultHashBuilder, LRUCache};
use database::Database;
use integer_encoding::VarInt;

use crate::{merkle::EMPTY_HASH, Error};

use super::Node;

#[derive(Debug)]
pub struct NodeDB<T> {
    db: T,
    cache: Mutex<LRUCache<[u8; 32], Node, DefaultHashBuilder>>,
}

const ROOTS_PREFIX: [u8; 1] = [1];
const NODES_PREFIX: [u8; 1] = [2];

// TODO: batch writes
// TODO: fast nodes
impl<T> NodeDB<T>
where
    T: Database,
{
    /// Panics if cache_size=0
    pub fn new(db: T, cache_size: usize) -> NodeDB<T> {
        assert!(cache_size > 0);
        NodeDB {
            db,
            cache: Mutex::new(
                LRUCache::new(cache_size).expect("won't panic since cache_size > zero"),
            ),
        }
    }

    pub fn get_versions(&self) -> BTreeSet<u32> {
        self.db
            .prefix_iterator(ROOTS_PREFIX.into())
            .map(|(k, _)| {
                u32::decode_var(&k)
                    .expect("invalid data in database - possible database corruption")
                    .0
            })
            .collect()
    }

    pub(crate) fn get_root_hash(&self, version: u32) -> Result<[u8; 32], Error> {
        self.db
            .get(&Self::get_root_key(version))
            .map(|hash| {
                hash.try_into()
                    .expect("invalid data in database - possible database corruption")
            })
            .ok_or(Error::VersionNotFound)
    }

    pub(crate) fn get_root_node(&self, version: u32) -> Result<Option<Node>, Error> {
        let root_hash = self.get_root_hash(version)?;

        if root_hash == EMPTY_HASH {
            return Ok(None);
        }

        Ok(Some(
            self.get_node(&root_hash)
                .expect("invalid data in database - possible database corruption"), // this node should be in the DB, if it isn't then better to panic
        ))
    }

    fn get_root_key(version: u32) -> Vec<u8> {
        [ROOTS_PREFIX.into(), version.encode_var_vec()].concat()
    }

    fn get_node_key(hash: &[u8; 32]) -> Vec<u8> {
        [NODES_PREFIX.to_vec(), hash.to_vec()].concat()
    }

    pub(crate) fn get_node(&self, hash: &[u8; 32]) -> Option<Node> {
        let cache = &mut self.cache.lock().expect("Lock will not be poisoned");
        let cache_node = cache.get(hash);

        if cache_node.is_some() {
            return cache_node.map(|v| v.to_owned());
        };

        let node_bytes = self.db.get(&Self::get_node_key(hash))?;
        let node = Node::deserialize(node_bytes)
            .expect("invalid data in database - possible database corruption");

        cache.put(*hash, node.clone());
        Some(node)
    }

    fn save_node(&mut self, node: &Node, hash: &[u8; 32]) {
        self.db.put(Self::get_node_key(hash), node.serialize());
        self.cache
            .lock()
            .expect("Lock will not be poisoned")
            .put(*hash, node.shallow_clone());
    }

    fn recursive_tree_save(&mut self, node: &Node, hash: &[u8; 32]) {
        if let Node::Inner(inner) = node {
            if let Some(left_node) = &inner.left_node {
                self.recursive_tree_save(left_node, &inner.left_hash);
            }
            if let Some(right_node) = &inner.right_node {
                self.recursive_tree_save(right_node, &inner.right_hash);
            }
        }

        self.save_node(node, hash)
    }

    /// Saves the given node and all of its descendants.
    /// Clears left_node/right_node on the root.
    pub(crate) fn save_tree(&mut self, root: &mut Node) -> [u8; 32] {
        let root_hash = root.hash();
        self.recursive_tree_save(root, &root_hash);

        if let Node::Inner(inner) = root {
            inner.left_node = None;
            inner.right_node = None;
        }

        root_hash
    }

    pub(crate) fn save_version(&mut self, version: u32, hash: &[u8; 32]) {
        let key = Self::get_root_key(version);
        self.db.put(key, hash.to_vec());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use database::MemDB;

    #[test]
    fn get_root_key_works() {
        let key = NodeDB::<MemDB>::get_root_key(1u32);
        assert_eq!(key, vec![1, 1])
    }

    #[test]
    fn get_node_key_works() {
        let key = NodeDB::<MemDB>::get_node_key(&[
            13, 181, 53, 227, 140, 38, 242, 22, 94, 152, 94, 71, 0, 89, 35, 122, 129, 85, 55, 190,
            253, 226, 35, 230, 65, 214, 244, 35, 69, 39, 223, 90,
        ]);
        assert_eq!(
            key,
            vec![
                2, 13, 181, 53, 227, 140, 38, 242, 22, 94, 152, 94, 71, 0, 89, 35, 122, 129, 85,
                55, 190, 253, 226, 35, 230, 65, 214, 244, 35, 69, 39, 223, 90
            ]
        )
    }

    #[test]
    fn get_versions_works() {
        let db = MemDB::new();
        db.put(NodeDB::<MemDB>::get_root_key(1u32), vec![]);
        let node_db = NodeDB {
            db,
            cache: Mutex::new(LRUCache::new(2).unwrap()),
        };

        let mut expected_versions = BTreeSet::new();
        expected_versions.insert(1);
        let versions = node_db.get_versions();

        assert_eq!(expected_versions, versions)
    }

    #[test]
    fn get_root_hash_works() {
        let root_hash = [
            13, 181, 53, 227, 140, 38, 242, 22, 94, 152, 94, 71, 0, 89, 35, 122, 129, 85, 55, 190,
            253, 226, 35, 230, 65, 214, 244, 35, 69, 39, 223, 90,
        ];
        let db = MemDB::new();
        db.put(NodeDB::<MemDB>::get_root_key(1u32), root_hash.into());
        let node_db = NodeDB {
            db,
            cache: Mutex::new(LRUCache::new(2).unwrap()),
        };

        let got_root_hash = node_db.get_root_hash(1).unwrap();

        assert_eq!(root_hash, got_root_hash);
    }
}
