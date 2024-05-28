use database::Database;

use super::{errors::GasStoreErrors, kv::GasKVStore};

pub mod mutable;

pub struct GasStorePrefix<'a, DB> {
    inner: GasKVStore<'a, DB>,
    prefix: Vec<u8>,
}

impl<'a, DB> GasStorePrefix<'a, DB> {
    pub(crate) fn new(inner: GasKVStore<'a, DB>, prefix: impl IntoIterator<Item = u8>) -> Self {
        Self {
            inner,
            prefix: prefix.into_iter().collect(),
        }
    }
}

impl<DB: Database> GasStorePrefix<'_, DB> {
    fn get<T: AsRef<[u8]> + ?Sized>(&mut self, k: &T) -> Result<Vec<u8>, GasStoreErrors> {
        let full_key = [&self.prefix, k.as_ref()].concat();
        self.inner.get(&full_key)
    }
}
