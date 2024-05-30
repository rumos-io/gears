use database::Database;
use kv_store::{
    ext::UnwrapInfallible, types::prefix::mutable::MutablePrefixStore, ReadPrefixStore,
    WritePrefixStore,
};

use crate::types::store::{errors::StoreErrors, gas::prefix::mutable::GasPrefixStoreMut};

use super::PrefixStore;

enum PrefixStoreMutBackend<'a, DB> {
    Gas(GasPrefixStoreMut<'a, DB>),
    Kv(MutablePrefixStore<'a, DB>),
}

pub struct PrefixStoreMut<'a, DB>(PrefixStoreMutBackend<'a, DB>);

impl<'a, DB> PrefixStoreMut<'a, DB> {
    pub fn to_immutable(&'a self) -> PrefixStore<'a, DB> {
        match &self.0 {
            PrefixStoreMutBackend::Gas(var) => {
                PrefixStore(super::PrefixStoreBackend::Gas(var.to_immutable()))
            }
            PrefixStoreMutBackend::Kv(var) => {
                PrefixStore(super::PrefixStoreBackend::Kv(var.to_immutable()))
            }
        }
    }
}

impl<'a, DB> From<GasPrefixStoreMut<'a, DB>> for PrefixStoreMut<'a, DB> {
    fn from(value: GasPrefixStoreMut<'a, DB>) -> Self {
        Self(PrefixStoreMutBackend::Gas(value))
    }
}

impl<'a, DB> From<MutablePrefixStore<'a, DB>> for PrefixStoreMut<'a, DB> {
    fn from(value: MutablePrefixStore<'a, DB>) -> Self {
        Self(PrefixStoreMutBackend::Kv(value))
    }
}

impl<DB: Database> ReadPrefixStore for PrefixStoreMut<'_, DB> {
    type Err = StoreErrors;

    fn get<T: AsRef<[u8]> + ?Sized>(&self, k: &T) -> Result<Option<Vec<u8>>, Self::Err> {
        match &self.0 {
            PrefixStoreMutBackend::Gas(var) => Ok(var.get(k)?),
            PrefixStoreMutBackend::Kv(var) => Ok(var.get(k).unwrap_infallible()),
        }
    }
}

impl<DB: Database> WritePrefixStore for PrefixStoreMut<'_, DB> {
    fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        k: KI,
        v: VI,
    ) -> Result<(), Self::Err> {
        match &mut self.0 {
            PrefixStoreMutBackend::Gas(var) => Ok(var.set(k, v)?),
            PrefixStoreMutBackend::Kv(var) => Ok(var.set(k, v).unwrap_infallible()),
        }
    }
}
