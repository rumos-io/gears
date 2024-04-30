use std::ops::{Bound, RangeBounds};

use database::Database;

use crate::{types::any::AnyKVStore, ReadPrefixStore};

use super::{prefix_end_bound, range::PrefixRange};

/// Wraps an immutable reference to a KVStore with a prefix
pub struct ImmutablePrefixStore<'a, DB> {
    pub(crate) store: AnyKVStore<'a, DB>,
    pub(crate) prefix: Vec<u8>,
}

impl<'a, DB: Database> ImmutablePrefixStore<'a, DB> {
    pub fn range<R: RangeBounds<Vec<u8>>>(&'a self, range: R) -> PrefixRange<'a, DB> {
        let new_start = match range.start_bound() {
            Bound::Included(b) => Bound::Included([self.prefix.clone(), b.clone()].concat()),
            Bound::Excluded(b) => Bound::Excluded([self.prefix.clone(), b.clone()].concat()),
            Bound::Unbounded => Bound::Included(self.prefix.clone()),
        };

        let new_end = match range.end_bound() {
            Bound::Included(b) => Bound::Included([self.prefix.clone(), b.clone()].concat()),
            Bound::Excluded(b) => Bound::Excluded([self.prefix.clone(), b.clone()].concat()),
            Bound::Unbounded => prefix_end_bound(self.prefix.clone()),
        };

        PrefixRange {
            parent_range: self.store.range((new_start, new_end)),
            prefix_length: self.prefix.len(),
        }
    }
}

impl<DB: Database> ReadPrefixStore for ImmutablePrefixStore<'_, DB> {
    fn get<T: AsRef<[u8]> + ?Sized>(&self, k: &T) -> Option<Vec<u8>> {
        let full_key = [&self.prefix, k.as_ref()].concat();
        self.store.get(&full_key)
    }
}
