use database::Database;
use store_crate::{types::prefix::mutable::MutablePrefixStore, ReadPrefixStore, WritePrefixStore};

use crate::types::store::{errors::GasStoreErrors, guard::GasGuard};

use super::GasStorePrefix;

pub struct GasStorePrefixMut<'a, DB> {
    inner: MutablePrefixStore<'a, DB>,
    guard: GasGuard<'a>,
}

impl<'a, DB> GasStorePrefixMut<'a, DB> {
    pub(crate) fn new(guard: GasGuard<'a>, inner: MutablePrefixStore<'a, DB>) -> Self {
        Self { inner, guard }
    }

    pub fn to_immutable(&'a self) -> GasStorePrefix<'a, DB> {
        GasStorePrefix {
            inner: self.inner.to_immutable(),
            guard: self.guard.clone(),
        }
    }
}

impl<DB: Database> ReadPrefixStore for GasStorePrefixMut<'_, DB> {
    type GetErr = GasStoreErrors;

    fn get<T: AsRef<[u8]> + ?Sized>(&self, k: &T) -> Result<Vec<u8>, Self::GetErr> {
        let value = self.inner.get(&k).ok();

        self.guard
            .get(k.as_ref().len(), value.as_ref().map(|this| this.len()))?;

        value.ok_or(GasStoreErrors::NotFound)
    }
}

impl<DB: Database> WritePrefixStore for GasStorePrefixMut<'_, DB> {
    type SetErr = GasStoreErrors;

    fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        k: KI,
        v: VI,
    ) -> Result<(), Self::SetErr> {
        let key = k.into_iter().collect::<Vec<_>>();
        let value = v.into_iter().collect::<Vec<_>>();

        self.guard.set(key.len(), value.len())?;

        self.inner.set(key, value).expect("Infallible");

        Ok(())
    }
}

impl<'a, DB: Database> GasStorePrefixMut<'a, DB> {
    pub fn delete(&mut self, k: &[u8]) -> Result<Option<Vec<u8>>, GasStoreErrors> {
        self.guard.delete()?;
        Ok(self.inner.delete(k))
    }
}
