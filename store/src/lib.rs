#![warn(rust_2018_idioms)]

use range::Range;
use strum::IntoEnumIterator;
use types::{
    kv::{mutable::KVStoreMut, KVStore},
    prefix::{immutable::ImmutablePrefixStore, mutable::MutablePrefixStore},
};

pub mod error;
mod hash;
pub mod range;
pub mod types;
mod utils;

pub mod database {
    pub use database::*;
}

use std::{hash::Hash, ops::RangeBounds};

pub(crate) const TREE_CACHE_SIZE: usize = 100_000;

pub trait StoreKey: Hash + Eq + IntoEnumIterator + Clone + Send + Sync + 'static {
    fn name(&self) -> &'static str;
}

pub trait ReadPrefixStore {
    fn get<T: AsRef<[u8]> + ?Sized>(&self, k: &T) -> Option<Vec<u8>>;
}

pub trait WritePrefixStore: ReadPrefixStore {
    fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(&mut self, k: KI, v: VI);
}

pub trait QueryableKVStore<DB> {
    fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>>;
    fn prefix_store<I: IntoIterator<Item = u8>>(&self, prefix: I) -> ImmutablePrefixStore<'_, DB>;
    fn range<R: RangeBounds<Vec<u8>> + Clone>(&self, range: R) -> Range<'_, R, DB>;
    // fn get_keys(&self, key_prefix: &(impl AsRef<[u8]> + ?Sized)) -> Vec<Vec<u8>>;
}

pub trait TransactionalKVStore<DB>: QueryableKVStore<DB> {
    fn prefix_store_mut(
        &mut self,
        prefix: impl IntoIterator<Item = u8>,
    ) -> MutablePrefixStore<'_, DB>;
    fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(&mut self, key: KI, value: VI);
}

pub trait QueryableMultiKVStore<DB, SK> {
    fn kv_store(&self, store_key: &SK) -> KVStore<'_, DB>;
    fn head_version(&self) -> u32;
    fn head_commit_hash(&self) -> [u8; 32];
}

pub trait TransactionalMultiKVStore<DB, SK> {
    fn kv_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, DB>;
    /// Writes then clears each store's tx cache to the store's block cache then clears the tx caches
    fn tx_cache_to_block(&mut self);
    /// Clears the tx caches
    fn tx_caches_clear(&mut self);
}
