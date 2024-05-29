use database::Database;
use kv_store::{
    ext::UnwrapInfallible, types::prefix::mutable::MutablePrefixStore, ReadPrefixStore,
    WritePrefixStore,
};

use crate::types::store::gas::{errors::GasStoreErrors, guard::GasGuard};

use super::GasPrefixStore;

pub struct GasPrefixStoreMut<'a, DB> {
    inner: MutablePrefixStore<'a, DB>,
    guard: GasGuard,
}

impl<'a, DB> GasPrefixStoreMut<'a, DB> {
    pub(crate) fn new(guard: GasGuard, inner: MutablePrefixStore<'a, DB>) -> Self {
        Self { inner, guard }
    }

    pub fn to_immutable(&'a self) -> GasPrefixStore<'a, DB> {
        GasPrefixStore {
            inner: self.inner.to_immutable(),
            guard: self.guard.clone(),
        }
    }
}

impl<DB: Database> ReadPrefixStore for GasPrefixStoreMut<'_, DB> {
    type Err = GasStoreErrors;

    fn get<T: AsRef<[u8]> + ?Sized>(&self, k: &T) -> Result<Option<Vec<u8>>, Self::Err> {
        let value = self.inner.get(&k).unwrap_infallible();

        self.guard
            .get(k.as_ref().len(), value.as_ref().map(|this| this.len()))?;

        Ok(value)
    }
}

impl<DB: Database> WritePrefixStore for GasPrefixStoreMut<'_, DB> {
    fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        k: KI,
        v: VI,
    ) -> Result<(), Self::Err> {
        let key = k.into_iter().collect::<Vec<_>>();
        let value = v.into_iter().collect::<Vec<_>>();

        self.guard.set(key.len(), value.len())?;

        self.inner.set(key, value).unwrap_infallible();

        Ok(())
    }
}

impl<'a, DB: Database> GasPrefixStoreMut<'a, DB> {
    pub fn delete(&mut self, k: &[u8]) -> Result<Option<Vec<u8>>, GasStoreErrors> {
        self.guard.delete()?;
        Ok(self.inner.delete(k))
    }
}
