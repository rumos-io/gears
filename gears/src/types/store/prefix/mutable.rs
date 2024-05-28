use database::Database;
use store_crate::{types::prefix::mutable::MutablePrefixStore, WritePrefixStore};

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

impl<'a, DB: Database> GasStorePrefixMut<'a, DB> {
    fn get<T: AsRef<[u8]>>(&'a self, k: &T) -> Result<Vec<u8>, GasStoreErrors> {
        self.to_immutable().get(k)
    }

    pub fn delete(&mut self, k: &[u8]) -> Result<Option<Vec<u8>>, GasStoreErrors> {
        self.guard.delete()?;
        Ok(self.inner.delete(k))
    }

    fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        k: KI,
        v: VI,
    ) -> Result<(), GasStoreErrors> {
        let key = k.into_iter().collect::<Vec<_>>();
        let value = v.into_iter().collect::<Vec<_>>();

        self.guard.set(key.len(), value.len())?;

        self.inner.set(key, value);

        Ok(())
    }
}
