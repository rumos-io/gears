use std::ops::RangeBounds;

use database::Database;
use kv_store::types::prefix::immutable::ImmutablePrefixStore;

use super::{errors::StoreErrors, gas::prefix::GasPrefixStore, range::StoreRange};

pub mod mutable;

enum PrefixStoreBackend<'a, DB> {
    Gas(GasPrefixStore<'a, DB>),
    Kv(ImmutablePrefixStore<'a, DB>),
}

pub struct PrefixStore<'a, DB>(pub(self) PrefixStoreBackend<'a, DB>);

impl<'a, DB> From<GasPrefixStore<'a, DB>> for PrefixStore<'a, DB> {
    fn from(value: GasPrefixStore<'a, DB>) -> Self {
        Self(PrefixStoreBackend::Gas(value))
    }
}

impl<'a, DB> From<ImmutablePrefixStore<'a, DB>> for PrefixStore<'a, DB> {
    fn from(value: ImmutablePrefixStore<'a, DB>) -> Self {
        Self(PrefixStoreBackend::Kv(value))
    }
}

impl<DB: Database> PrefixStore<'_, DB> {
    pub fn range<R: RangeBounds<Vec<u8>> + Clone>(&self, range: R) -> StoreRange<'_, DB> {
        match &self.0 {
            PrefixStoreBackend::Gas(var) => var.range(range).into(),
            PrefixStoreBackend::Kv(var) => var.range(range).into(),
        }
    }

    pub fn get<T: AsRef<[u8]> + ?Sized>(&self, k: &T) -> Result<Option<Vec<u8>>, StoreErrors> {
        match &self.0 {
            PrefixStoreBackend::Gas(var) => Ok(var.get(k)?),
            PrefixStoreBackend::Kv(var) => Ok(var.get(k)),
        }
    }
}
