use database::Database;
use store_crate::{types::kv::mutable::KVStoreMut, TransactionalKVStore};

use crate::types::{
    auth::gas::Gas,
    gas::{config::GasConfig, kind::TxKind, GasMeter},
    store::constants::DELETE_DESC,
};

use super::{
    constants::{WRITE_COST_FLAT_DESC, WRITE_PER_BYTE_DESC},
    errors::GasStoreErrors,
    kv::GasKVStore,
};

#[derive(Debug)]
pub struct GasKVStoreMut<'a, DB> {
    pub(super) gas_meter: &'a mut GasMeter<TxKind>,
    pub(super) inner: KVStoreMut<'a, DB>,
}

impl<'a, DB> GasKVStoreMut<'a, DB> {
    pub fn new(gas_meter: &'a mut GasMeter<TxKind>, inner: KVStoreMut<'a, DB>) -> Self {
        Self { gas_meter, inner }
    }

    pub fn to_immutable(&mut self) -> GasKVStore<'_, DB> {
        GasKVStore {
            gas_meter: &mut *self.gas_meter,
            inner: self.inner.to_immutable(),
        }
    }
}

impl<DB: Database> GasKVStoreMut<'_, DB> {
    pub fn get<R: AsRef<[u8]>>(&mut self, k: R) -> Result<Vec<u8>, GasStoreErrors> {
        self.to_immutable().get(k)
    }

    fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        key: KI,
        value: VI,
    ) -> Result<(), GasStoreErrors> {
        self.gas_meter
            .consume_gas(GasConfig::kv().read_cost_flat, WRITE_COST_FLAT_DESC)?;

        let key = key.into_iter().collect::<Vec<_>>();
        let value = value.into_iter().collect::<Vec<_>>();

        let read_cost_per_byte = GasConfig::kv().write_cost_per_byte;

        self.gas_meter.consume_gas(
            read_cost_per_byte
                .checked_mul(Gas::try_from(key.len() as u64)?)
                .ok_or(GasStoreErrors::GasOverflow)?,
            WRITE_PER_BYTE_DESC,
        )?;

        self.gas_meter.consume_gas(
            read_cost_per_byte
                .checked_mul(Gas::try_from(value.len() as u64)?)
                .ok_or(GasStoreErrors::GasOverflow)?,
            WRITE_PER_BYTE_DESC,
        )?;

        self.inner.set(key, value);

        Ok(())
    }

    pub fn delete(&mut self, k: &[u8]) -> Result<Vec<u8>, GasStoreErrors> {
        self.gas_meter
            .consume_gas(GasConfig::kv().delete_cost, DELETE_DESC)?;

        self.inner.delete(k).ok_or(GasStoreErrors::NotFound)
    }
}
