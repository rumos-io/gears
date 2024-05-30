use database::Database;
use kv_store::{ext::UnwrapInfallible, types::kv::immutable::KVStore, QueryableKVStore};

use super::{errors::StoreErrors, gas::kv::GasKVStore, prefix::PrefixStore, range::StoreRange};

pub mod mutable;

pub enum StoreBackend<'a, DB> {
    Gas(GasKVStore<'a, DB>),
    Kv(KVStore<'a, DB>),
}

pub struct Store<'a, DB>(StoreBackend<'a, DB>);

impl<'a, DB> From<GasKVStore<'a, DB>> for Store<'a, DB> {
    fn from(value: GasKVStore<'a, DB>) -> Self {
        Self(StoreBackend::Gas(value))
    }
}

impl<'a, DB> From<KVStore<'a, DB>> for Store<'a, DB> {
    fn from(value: KVStore<'a, DB>) -> Self {
        Self(StoreBackend::Kv(value))
    }
}

impl<'a, DB: Database> QueryableKVStore for Store<'a, DB> {
    type Prefix = PrefixStore<'a, DB>;

    type Range = StoreRange<'a, DB>;

    type Err = StoreErrors;

    fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Result<Option<Vec<u8>>, Self::Err> {
        match &self.0 {
            StoreBackend::Gas(var) => Ok(var.get(k)?),
            StoreBackend::Kv(var) => Ok(var.get(k).unwrap_infallible()),
        }
    }

    fn prefix_store<I: IntoIterator<Item = u8>>(self, prefix: I) -> Self::Prefix {
        match self.0 {
            StoreBackend::Gas(var) => var.prefix_store(prefix).into(),
            StoreBackend::Kv(var) => var.prefix_store(prefix).into(),
        }
    }

    fn range<R: std::ops::RangeBounds<Vec<u8>> + Clone>(&self, range: R) -> Self::Range {
        match &self.0 {
            StoreBackend::Gas(var) => StoreRange::from(var.range(range)),
            StoreBackend::Kv(var) => StoreRange::from(var.range(range)),
        }
    }
}
