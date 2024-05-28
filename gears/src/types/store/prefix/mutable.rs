use database::Database;
use store_crate::types::prefix::mutable::MutablePrefixStore;

use crate::types::gas::{kind::TxKind, GasMeter};

pub struct GasStorePrefixMut<'a, DB> {
    gas_meter: &'a mut GasMeter<TxKind>,
    inner: MutablePrefixStore<'a, DB>,
}

impl<'a, DB> GasStorePrefixMut<'a, DB> {
    pub fn new(gas_meter: &'a mut GasMeter<TxKind>, inner: MutablePrefixStore<'a, DB>) -> Self {
        Self { gas_meter, inner }
    }
}

impl<DB: Database> GasStorePrefixMut<'_, DB> {}
