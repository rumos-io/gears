use std::{borrow::Cow, ops::RangeBounds};

use database::Database;
use store_crate::range::Range;

use crate::types::{
    auth::gas::Gas,
    gas::{config::GasConfig, kind::TxKind, GasMeter},
};

use super::constants::{ITER_NEXT_CAST_FLAT_DESC, VALUE_PER_BYTE_DESC};

pub struct GasRange<'a, R: RangeBounds<Vec<u8>>, DB> {
    inner: Range<'a, R, DB>,
    gas_meter: &'a mut GasMeter<TxKind>,
}

impl<'a, R: RangeBounds<Vec<u8>>, DB> GasRange<'a, R, DB> {
    pub(super) fn new(inner: Range<'a, R, DB>, gas_meter: &'a mut GasMeter<TxKind>) -> Self {
        Self { inner, gas_meter }
    }
}

impl<'a, R: RangeBounds<Vec<u8>>, DB: Database> Iterator for GasRange<'a, R, DB> {
    type Item = (Cow<'a, Vec<u8>>, Cow<'a, Vec<u8>>);

    // TODO:NOW What to do with all this error handling?
    fn next(&mut self) -> Option<Self::Item> {
        let next = if let Some((key, value)) = self.inner.next() {
            let read_cost_per_byte = GasConfig::kv().read_cost_per_byte;

            self.gas_meter
                .consume_gas(
                    read_cost_per_byte.checked_mul(Gas::try_from(key.len() as u64).ok()?)?,
                    VALUE_PER_BYTE_DESC,
                )
                .ok()?;

            self.gas_meter
                .consume_gas(
                    read_cost_per_byte.checked_mul(Gas::try_from(value.len() as u64).ok()?)?,
                    VALUE_PER_BYTE_DESC,
                )
                .ok()?;
            Some((key, value))
        } else {
            None
        };

        self.gas_meter
            .consume_gas(
                GasConfig::kv().iter_next_cost_flat,
                ITER_NEXT_CAST_FLAT_DESC,
            )
            .ok()?;

        next
    }
}
