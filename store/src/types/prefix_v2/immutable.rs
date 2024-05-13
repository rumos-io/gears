use std::ops::{Bound, RangeBounds};

use database::Database;

use crate::{types::kv_2::immutable::KVStoreV2, QueryableKVStoreV2, ReadPrefixStore};

use super::{prefix_end_bound, range::PrefixRange};

pub struct ImmutablePrefixStoreV2<'a, DB> {
    pub(crate) store: KVStoreV2<'a, DB>,
    pub(crate) prefix: Vec<u8>,
}

impl<'a, DB: Database> ImmutablePrefixStoreV2<'a, DB> {
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

impl<DB: Database> ReadPrefixStore for ImmutablePrefixStoreV2<'_, DB> {
    fn get<T: AsRef<[u8]> + ?Sized>(&self, k: &T) -> Option<Vec<u8>> {
        let full_key = [&self.prefix, k.as_ref()].concat();
        self.store.get(&full_key)
    }
}
