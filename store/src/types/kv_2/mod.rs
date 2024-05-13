use std::{
    borrow::Cow,
    marker::PhantomData,
    ops::RangeBounds,
    sync::{Arc, RwLock},
};

use database::Database;
use trees::iavl::Tree;

use crate::{
    error::{StoreError, POISONED_LOCK},
    range::Range,
    utils::MergedRange,
    TREE_CACHE_SIZE,
};

use self::store_cache::KVStoreCacheV2;

pub mod cache;
pub mod commit;
pub mod immutable;
pub mod mutable;
pub mod store_cache;

pub struct KVStorage<DB, SK> {
    persistent: Arc<RwLock<Tree<DB>>>,
    cache: KVStoreCacheV2,
    _marker: PhantomData<SK>,
}

impl<DB: Database, SK> KVStorage<DB, SK> {
    pub fn new(db: DB, target_version: Option<u32>) -> Result<Self, StoreError> {
        Ok(Self {
            persistent: Arc::new(RwLock::new(Tree::new(
                db,
                target_version,
                TREE_CACHE_SIZE
                    .try_into()
                    .expect("Unreachable. Tree cache size is > 0"),
            )?)),
            cache: Default::default(),
            _marker: PhantomData,
        })
    }

    #[inline]
    pub fn head_commit_hash(&self) -> [u8; 32] {
        self.persistent.read().expect(POISONED_LOCK).root_hash()
    }

    #[inline]
    pub fn last_committed_version(&self) -> u32 {
        self.persistent
            .read()
            .expect(POISONED_LOCK)
            .loaded_version()
    }

    #[inline]
    pub fn delete(&mut self, k: &[u8]) -> Option<Vec<u8>> {
        self.cache
            .delete(k)
            .or(self.persistent.read().expect(POISONED_LOCK).get(k))
    }

    #[inline]
    pub fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        key: KI,
        value: VI,
    ) {
        self.cache.set(key, value)
    }

    pub fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>> {
        self.cache.get(k.as_ref()).cloned().or(self
            .persistent
            .read()
            .expect(POISONED_LOCK)
            .get(k.as_ref()))
    }

    // TODO:NOW You could iterate over values that should have been deleted
    pub fn range<R: RangeBounds<Vec<u8>> + Clone>(&self, range: R) -> Range<'_, R, DB> {
        let cached_values = self
            .cache
            .storage
            .range(range.clone())
            .into_iter()
            .map(|(first, second)| (Cow::Borrowed(first), Cow::Borrowed(second)));

        let tree = self.persistent.read().expect(POISONED_LOCK);
        let persisted_values = tree
            .range(range)
            .map(|(first, second)| (Cow::Owned(first), Cow::Owned(second)));

        MergedRange::merge(cached_values, persisted_values).into()
    }
}
