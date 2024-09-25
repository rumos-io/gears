use database::{prefix::PrefixDB, Database};
use std::{collections::HashMap, hash::Hash, sync::Arc};
use strum::IntoEnumIterator;

pub mod bank;
pub mod cache;
pub mod error;
pub mod query;
pub mod range;
pub mod store;

mod hash;
mod utils;

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

fn build_prefixed_stores<DB: Database, SK: StoreKey>(db: Arc<DB>) -> HashMap<SK, PrefixDB<DB>> {
    let mut stores = HashMap::new();

    for store in SK::iter() {
        let prefix = store.name().as_bytes().to_vec();
        let prefixed_db = PrefixDB::new(Arc::clone(&db), prefix);

        stores.insert(store, prefixed_db);
    }

    stores
}

// pub trait ReadPrefixStore {
//     type Err;

//     fn get<T: AsRef<[u8]> + ?Sized>(&self, k: &T) -> Result<Option<Vec<u8>>, Self::Err>;
// }

// pub trait WritePrefixStore: ReadPrefixStore {
//     fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
//         &mut self,
//         k: KI,
//         v: VI,
//     ) -> Result<(), Self::Err>;
// }

// pub trait QueryableKVStore {
//     type Prefix: ReadPrefixStore;
//     type Err;

//     fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Result<Option<Vec<u8>>, Self::Err>;
//     fn prefix_store<I: IntoIterator<Item = u8>>(self, prefix: I) -> Self::Prefix;
//     // fn range(&self, range: (Bound<Vec<u8>>, Bound<Vec<u8>>)) -> Self::Range; // Borrow checker won't work for mutable version of it
//     // fn get_keys(&self, key_prefix: &(impl AsRef<[u8]> + ?Sized)) -> Vec<Vec<u8>>;
// }

// pub trait TransactionalKVStore: QueryableKVStore {
//     type PrefixMut: WritePrefixStore;

//     fn prefix_store_mut(self, prefix: impl IntoIterator<Item = u8>) -> Self::PrefixMut;
//     fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
//         &mut self,
//         key: KI,
//         value: VI,
//     ) -> Result<(), Self::Err>;
// }

// pub trait QueryableMultiKVStore<DB, SK> {
//     fn kv_store(&self, store_key: &SK) -> KVStore<'_, DB>;
//     fn head_version(&self) -> u32;
//     fn head_commit_hash(&self) -> [u8; 32];
// }

// pub trait TransactionalMultiKVStore<DB, SK> {
//     fn kv_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, DB>;
//     /// Clears the tx caches
//     fn caches_clear(&mut self);
// }
