pub mod mutable;

use std::{
    cell::RefCell,
    ops::{Bound, RangeBounds},
    sync::Arc,
};

use database::Database;
use store_crate::{ext::UnwrapInfallible, types::kv::immutable::KVStore, QueryableKVStore};

use crate::types::gas::{kind::TxKind, GasMeter};

use super::{errors::GasStoreErrors, guard::GasGuard, prefix::GasStorePrefix, range::GasRange};

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

impl<'a, DB: Database> QueryableKVStore for GasKVStore<'a, DB> {
    type Prefix = GasStorePrefix<'a, DB>;

    type Range = GasRange<'a, (Bound<Vec<u8>>, Bound<Vec<u8>>), DB>;

    type GetErr = GasStoreErrors;

    fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Result<Option<Vec<u8>>, Self::GetErr> {
        let value = self.inner.get(&k).infallible();

        self.guard
            .get(k.as_ref().len(), value.as_ref().map(|this| this.len()))?;

        Ok(value)
    }

    fn prefix_store<I: IntoIterator<Item = u8>>(self, prefix: I) -> Self::Prefix {
        GasStorePrefix::new(self.guard, self.inner.prefix_store(prefix))
    }

    fn range<R: RangeBounds<Vec<u8>> + Clone>(&self, _range: R) -> Self::Range {
        // GasRange::new_kv(self.inner.range(range), self.guard.clone())
        todo!()
    }
}
