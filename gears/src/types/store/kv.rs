use std::cell::RefCell;

use database::Database;
use store_crate::{types::kv::immutable::KVStore, QueryableKVStore};

use crate::types::{
    auth::gas::Gas,
    gas::{config::GasConfig, kind::TxKind, GasMeter},
};

use super::{
    constants::{READ_COST_FLAT_DESC, READ_PER_BYTE_DESC},
    errors::GasStoreErrors,
};

#[derive(Debug)]
pub struct GasKVStore<'a, DB> {
    gas_meter: RefCell<GasMeter<TxKind>>,
    inner: KVStore<'a, DB>,
}

impl<'a, DB> GasKVStore<'a, DB> {
    pub fn new(gas_meter: GasMeter<TxKind>, inner: KVStore<'a, DB>) -> Self {
        Self {
            gas_meter: RefCell::new(gas_meter),
            inner,
        }
    }
}

impl<DB: Database> GasKVStore<'_, DB> {
    fn get<R: AsRef<[u8]>>(&self, k: R) -> Result<Vec<u8>, GasStoreErrors> //Option<Vec<u8>>
    {
        self.gas_meter
            .borrow_mut()
            .consume_gas(GasConfig::kv().read_cost_flat, READ_COST_FLAT_DESC)?;

        let value = self.inner.get(&k);

        let read_cost_per_byte = GasConfig::kv().read_cost_per_byte;

        let mut gas_mut = self.gas_meter.borrow_mut();

        gas_mut.consume_gas(
            read_cost_per_byte
                .checked_add(Gas::try_from(k.as_ref().len() as u64)?)
                .ok_or(GasStoreErrors::GasOverflow)?,
            READ_PER_BYTE_DESC,
        )?;

        // TODO:NOW is it okay to ignore if value not found. Don't see any other way
        if let Some(ref value) = value {
            gas_mut.consume_gas(
                read_cost_per_byte
                    .checked_add(Gas::try_from(value.len() as u64)?)
                    .ok_or(GasStoreErrors::GasOverflow)?,
                READ_PER_BYTE_DESC,
            )?;
        }

        value.ok_or(GasStoreErrors::NotFound)
    }
}
