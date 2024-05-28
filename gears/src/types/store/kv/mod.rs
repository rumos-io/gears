pub mod mutable;

use std::{cell::RefCell, sync::Arc};

use database::Database;
use store_crate::{types::kv::immutable::KVStore, QueryableKVStore};

use crate::types::gas::{kind::TxKind, GasMeter};

use super::{errors::GasStoreErrors, guard::GasGuard, prefix::GasStorePrefix};

#[derive(Debug)]
pub struct GasKVStore<'a, DB> {
    pub(super) guard: GasGuard<'a>,
    pub(super) inner: KVStore<'a, DB>,
}

impl<'a, DB> GasKVStore<'a, DB> {
    pub(crate) fn new(guard: &'a mut GasMeter<TxKind>, inner: KVStore<'a, DB>) -> Self {
        Self {
            guard: GasGuard(Arc::new(RefCell::new(guard))),
            inner,
        }
    }
}

impl<'a, DB: Database> GasKVStore<'a, DB> {
    pub fn get<R: AsRef<[u8]>>(&self, k: R) -> Result<Vec<u8>, GasStoreErrors> {
        let value = self.inner.get(&k);

        self.guard
            .get(k.as_ref().len(), value.as_ref().map(|this| this.len()))?;

        value.ok_or(GasStoreErrors::NotFound)
    }

    pub fn prefix_store<I: IntoIterator<Item = u8>>(self, prefix: I) -> GasStorePrefix<'a, DB> {
        GasStorePrefix::new(self.guard, self.inner.prefix_store(prefix))
    }

    // pub fn range<R: RangeBounds<Vec<u8>> + Clone>(&mut self, range: R) -> GasRange<'_, R, DB> {
    //     GasRange::new(self.inner.range(range), &mut self.gas_meter)
    // }
}
