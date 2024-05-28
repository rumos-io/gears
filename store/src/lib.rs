use strum::IntoEnumIterator;
use types::kv::{immutable::KVStore, mutable::KVStoreMut};

pub mod error;
pub mod ext;
mod hash;
pub mod range;
pub mod types;
mod utils;

use std::{hash::Hash, ops::RangeBounds};

pub(crate) const TREE_CACHE_SIZE: usize = 100_000;

#[derive(Debug, Clone, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct TransactionStore;
#[derive(Debug, Clone, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ApplicationStore;

pub trait StoreKey:
    std::fmt::Debug + Hash + Eq + IntoEnumIterator + Clone + Send + Sync + 'static
{
    fn name(&self) -> &'static str;

    /// Return key for parameters
    fn params() -> &'static Self;
}

pub trait ReadPrefixStore {
    type GetErr;

    fn get<T: AsRef<[u8]> + ?Sized>(&self, k: &T) -> Result<Option<Vec<u8>>, Self::GetErr>;
}

pub trait WritePrefixStore: ReadPrefixStore {
    type SetErr;

    fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        k: KI,
        v: VI,
    ) -> Result<(), Self::SetErr>;
}

pub trait QueryableKVStore {
    type Prefix: ReadPrefixStore;
    type Range: Iterator;
    type GetErr;

    fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Result<Option<Vec<u8>>, Self::GetErr>;
    fn prefix_store<I: IntoIterator<Item = u8>>(self, prefix: I) -> Self::Prefix;
    fn range<R: RangeBounds<Vec<u8>> + Clone>(&self, range: R) -> Self::Range;
    // fn get_keys(&self, key_prefix: &(impl AsRef<[u8]> + ?Sized)) -> Vec<Vec<u8>>;
}

pub trait TransactionalKVStore: QueryableKVStore {
    type PrefixMut: WritePrefixStore;
    type SetErr;

    fn prefix_store_mut(self, prefix: impl IntoIterator<Item = u8>) -> Self::PrefixMut;
    fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        key: KI,
        value: VI,
    ) -> Result<(), Self::SetErr>;
}

pub trait QueryableMultiKVStore<DB, SK> {
    fn kv_store(&self, store_key: &SK) -> KVStore<'_, DB>;
    fn head_version(&self) -> u32;
    fn head_commit_hash(&self) -> [u8; 32];
}

pub trait TransactionalMultiKVStore<DB, SK> {
    fn kv_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, DB>;
    /// Clears the tx caches
    fn caches_clear(&mut self);
}
