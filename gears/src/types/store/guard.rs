use std::{cell::RefCell, sync::Arc};

use crate::types::{
    auth::gas::Gas,
    gas::{config::GasConfig, kind::TxKind, GasMeter},
};

use super::{
    constants::{
        DELETE_DESC, ITER_NEXT_CAST_FLAT_DESC, READ_COST_FLAT_DESC, READ_PER_BYTE_DESC,
        VALUE_PER_BYTE_DESC, WRITE_COST_FLAT_DESC, WRITE_PER_BYTE_DESC,
    },
    errors::GasStoreErrors,
};

#[derive(Debug, Clone)]
pub struct GasGuard(pub(super) Arc<RefCell<GasMeter<TxKind>>>);

impl GasGuard {
    pub(crate) fn new(inner: Arc<RefCell<GasMeter<TxKind>>>) -> Self {
        Self(inner)
    }

    pub fn get(&self, key: usize, value: Option<usize>) -> Result<(), GasStoreErrors> {
        let mut gas_meter = self.0.borrow_mut();

        let read_cost_per_byte = GasConfig::kv().read_cost_per_byte;

        gas_meter.consume_gas(GasConfig::kv().read_cost_flat, READ_COST_FLAT_DESC)?;

        gas_meter.consume_gas(
            read_cost_per_byte
                .checked_mul(Gas::try_from(key as u64)?)
                .ok_or(GasStoreErrors::GasOverflow)?,
            READ_PER_BYTE_DESC,
        )?;

        // TODO:NOW is it okay to ignore if value not found. Don't see any other way
        if let Some(value) = value {
            gas_meter.consume_gas(
                read_cost_per_byte
                    .checked_mul(Gas::try_from(value as u64)?)
                    .ok_or(GasStoreErrors::GasOverflow)?,
                READ_PER_BYTE_DESC,
            )?;
        }

        Ok(())
    }

    pub fn set(&self, key: usize, value: usize) -> Result<(), GasStoreErrors> {
        let mut gas_meter = self.0.borrow_mut();
        gas_meter.consume_gas(GasConfig::kv().read_cost_flat, WRITE_COST_FLAT_DESC)?;

        let write_cost_per_byte = GasConfig::kv().write_cost_per_byte;

        gas_meter.consume_gas(
            write_cost_per_byte
                .checked_mul(Gas::try_from(key as u64)?)
                .ok_or(GasStoreErrors::GasOverflow)?,
            WRITE_PER_BYTE_DESC,
        )?;

        gas_meter.consume_gas(
            write_cost_per_byte
                .checked_mul(Gas::try_from(value as u64)?)
                .ok_or(GasStoreErrors::GasOverflow)?,
            WRITE_PER_BYTE_DESC,
        )?;

        Ok(())
    }

    pub fn delete(&self) -> Result<(), GasStoreErrors> {
        self.0
            .borrow_mut()
            .consume_gas(GasConfig::kv().delete_cost, DELETE_DESC)?;

        Ok(())
    }

    pub fn range(&self, key_value: Option<(usize, usize)>) -> Result<(), GasStoreErrors> {
        let mut gas_meter = self.0.borrow_mut();

        if let Some((key, value)) = key_value {
            let read_cost_per_byte = GasConfig::kv().read_cost_per_byte;

            gas_meter.consume_gas(
                read_cost_per_byte
                    .checked_mul(Gas::try_from(key as u64)?)
                    .ok_or(GasStoreErrors::GasOverflow)?,
                VALUE_PER_BYTE_DESC,
            )?;

            gas_meter.consume_gas(
                read_cost_per_byte
                    .checked_mul(Gas::try_from(value as u64)?)
                    .ok_or(GasStoreErrors::GasOverflow)?,
                VALUE_PER_BYTE_DESC,
            )?;
        }

        gas_meter.consume_gas(
            GasConfig::kv().iter_next_cost_flat,
            ITER_NEXT_CAST_FLAT_DESC,
        )?;

        Ok(())
    }
}
