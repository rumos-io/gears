pub mod mutable;

use std::ops::RangeBounds;

use database::Database;
use kv_store::{ext::UnwrapInfallible, types::kv::immutable::KVStore, QueryableKVStore};

use super::{errors::GasStoreErrors, guard::GasGuard, prefix::GasPrefixStore, range::GasRange};

#[derive(Debug)]
pub struct GasKVStore<'a, DB> {
    pub(super) guard: Option<GasGuard>,
    pub(super) inner: KVStore<'a, DB>,
}

impl<'a, DB> GasKVStore<'a, DB> {
    pub(crate) fn new(guard: Option<GasGuard>, inner: KVStore<'a, DB>) -> Self {
        Self { guard, inner }
    }
}

impl<'a, DB: Database> QueryableKVStore for GasKVStore<'a, DB> {
    type Prefix = GasPrefixStore<'a, DB>;

    type Range = GasRange<'a, DB>;

    type Err = GasStoreErrors;

    fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Result<Option<Vec<u8>>, Self::Err> {
        let value = self.inner.get(&k).unwrap_infallible();

        if let Some(guard) = &self.guard {
            guard.get(k.as_ref().len(), value.as_ref().map(|this| this.len()))?;
        }

        Ok(value)
    }

    fn prefix_store<I: IntoIterator<Item = u8>>(self, prefix: I) -> Self::Prefix {
        GasPrefixStore::new(self.guard, self.inner.prefix_store(prefix))
    }

    fn range<R: RangeBounds<Vec<u8>> + Clone>(&self, _range: R) -> Self::Range {
        // GasRange::new_kv(self.inner.range(range), self.guard.clone())
        todo!()
    }
}
