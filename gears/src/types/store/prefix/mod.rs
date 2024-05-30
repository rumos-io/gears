use database::Database;
use kv_store::{
    ext::UnwrapInfallible, types::prefix::immutable::ImmutablePrefixStore, ReadPrefixStore,
};

use super::{errors::StoreErrors, gas::prefix::GasPrefixStore};

pub mod mutable;

enum PrefixStoreBackend<'a, DB> {
    Gas(GasPrefixStore<'a, DB>),
    Kv(ImmutablePrefixStore<'a, DB>),
}

pub struct PrefixStore<'a, DB>(PrefixStoreBackend<'a, DB>);

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

impl<DB: Database> ReadPrefixStore for PrefixStore<'_, DB> {
    type Err = StoreErrors;

    fn get<T: AsRef<[u8]> + ?Sized>(&self, k: &T) -> Result<Option<Vec<u8>>, Self::Err> {
        match &self.0 {
            PrefixStoreBackend::Gas(var) => Ok(var.get(k)?),
            PrefixStoreBackend::Kv(var) => Ok(var.get(k).unwrap_infallible()),
        }
    }
}
