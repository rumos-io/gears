use ::database::PrefixDB;
use range::Range;
use strum::IntoEnumIterator;
use types::{
    kv_2::{immutable::KVStoreV2, mutable::KVStoreMutV2},
    prefix_v2::{immutable::ImmutablePrefixStoreV2, mutable::MutablePrefixStoreV2},
};

pub mod error;
mod hash;
pub mod range;
pub mod types;
mod utils;

use std::{hash::Hash, ops::RangeBounds};

pub(crate) const TREE_CACHE_SIZE: usize = 100_000;

pub struct CacheKind;
pub struct CommitKind;

pub trait StoreKey: Hash + Eq + IntoEnumIterator + Clone + Send + Sync + 'static {
    fn name(&self) -> &'static str;
}

pub trait ReadPrefixStore {
    fn get<T: AsRef<[u8]> + ?Sized>(&self, k: &T) -> Option<Vec<u8>>;
}

pub trait WritePrefixStore: ReadPrefixStore {
    fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(&mut self, k: KI, v: VI);
}

// pub trait QueryableKVStore<'a, DB> {
//     fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>>;
//     fn prefix_store<I: IntoIterator<Item = u8>>(self, prefix: I) -> ImmutablePrefixStore<'a, DB>;
//     fn range<R: RangeBounds<Vec<u8>> + Clone>(&self, range: R) -> Range<'_, R, DB>;
//     // fn get_keys(&self, key_prefix: &(impl AsRef<[u8]> + ?Sized)) -> Vec<Vec<u8>>;
// }

pub trait QueryableKVStoreV2<'a, DB> {
    fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>>;
    fn prefix_store<I: IntoIterator<Item = u8>>(self, prefix: I) -> ImmutablePrefixStoreV2<'a, DB>;
    fn range<R: RangeBounds<Vec<u8>> + Clone>(&self, range: R) -> Range<'_, R, DB>;
    // fn get_keys(&self, key_prefix: &(impl AsRef<[u8]> + ?Sized)) -> Vec<Vec<u8>>;
}

// pub trait TransactionalKVStore<'a, DB>: QueryableKVStore<'a, DB> {
//     fn prefix_store_mut(self, prefix: impl IntoIterator<Item = u8>) -> MutablePrefixStore<'a, DB>;
//     fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(&mut self, key: KI, value: VI);
// }

pub trait TransactionalKVStoreV2<'a, DB>: QueryableKVStoreV2<'a, DB> {
    fn prefix_store_mut(self, prefix: impl IntoIterator<Item = u8>)
        -> MutablePrefixStoreV2<'a, DB>;
    fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(&mut self, key: KI, value: VI);
}

pub trait QueryableMultiKVStoreV2<DB, SK> {
    fn kv_store(&self, store_key: &SK) -> KVStoreV2<'_, DB>;
    fn head_version(&self) -> u32;
    fn head_commit_hash(&self) -> [u8; 32];
}

// pub trait TransactionalMultiKVStore<DB, SK> {
//     fn kv_store_mut(&mut self, store_key: &SK) -> KVStoreMutV2<'_, PrefixDB<DB>>;
//     /// Writes then clears each store's tx cache to the store's block cache then clears the tx caches
//     fn tx_cache_to_block(&mut self);
//     /// Clears the tx caches
//     fn tx_caches_clear(&mut self);
// }

pub trait TransactionalMultiKVStoreV2<DB, SK> {
    fn kv_store_mut(&mut self, store_key: &SK) -> KVStoreMutV2<'_, PrefixDB<DB>>;
    /// Clears the tx caches
    fn caches_clear(&mut self);
}
