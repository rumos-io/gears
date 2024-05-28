use database::Database;
use store_crate::types::prefix::immutable::ImmutablePrefixStore;

use crate::types::gas::{kind::TxKind, GasMeter};

pub mod mutable;

pub struct GasStorePrefix<'a, DB> {
    gas_meter: &'a mut GasMeter<TxKind>,
    inner: ImmutablePrefixStore<'a, DB>,
}

impl<'a, DB> GasStorePrefix<'a, DB> {
    pub fn new(gas_meter: &'a mut GasMeter<TxKind>, inner: ImmutablePrefixStore<'a, DB>) -> Self {
        Self { gas_meter, inner }
    }
}

impl<DB: Database> GasStorePrefix<'_, DB> {}
