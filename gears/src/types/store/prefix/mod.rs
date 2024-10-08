use std::ops::{Bound, RangeBounds};

use database::Database;
use kv_store::store::prefix::immutable::ImmutablePrefixStore;

use super::{
    gas::{errors::GasStoreErrors, prefix::GasPrefixStore},
    range::StoreRange,
};

pub mod mutable;

#[derive(Debug, Clone)]
enum PrefixStoreBackend<'a, DB> {
    Gas(GasPrefixStore<'a, DB>),
    Kv(ImmutablePrefixStore<'a, DB>),
}

#[derive(Debug, Clone)]
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

impl<'a, DB: Database> PrefixStore<'a, DB> {
    pub fn into_range<R: RangeBounds<Vec<u8>> + Clone>(
        self,
        range: R,
    ) -> StoreRange<'a, DB, Vec<u8>, (Bound<Vec<u8>>, Bound<Vec<u8>>)> {
        match self.0 {
            PrefixStoreBackend::Gas(var) => var.into_range(range).into(),
            PrefixStoreBackend::Kv(var) => var.into_range(range).into(),
        }
    }
}

impl<DB: Database> PrefixStore<'_, DB> {
    pub fn get<T: AsRef<[u8]> + ?Sized>(&self, k: &T) -> Result<Option<Vec<u8>>, GasStoreErrors> {
        match &self.0 {
            PrefixStoreBackend::Gas(var) => Ok(var.get(k)?),
            PrefixStoreBackend::Kv(var) => Ok(var.get(k)),
        }
    }
}
