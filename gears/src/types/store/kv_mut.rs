use store_crate::types::kv::immutable::KVStore;

use crate::types::gas::{kind::TxKind, GasMeter};

#[derive(Debug)]
pub struct GasKVStoreMut<'a, DB> {
    gas_meter: GasMeter<TxKind>,
    inner: KVStore<'a, DB>,
}

impl<'a, DB> GasKVStoreMut<'a, DB> {
    pub fn new(gas_meter: GasMeter<TxKind>, inner: KVStore<'a, DB>) -> Self {
        Self { gas_meter, inner }
    }
}
