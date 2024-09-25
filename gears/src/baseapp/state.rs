use database::Database;
use kv_store::bank::multi::ApplicationMultiBank;

use crate::{
    application::handlers::node::ABCIHandler,
    types::gas::{basic_meter::BasicGasMeter, infinite_meter::InfiniteGasMeter, Gas, GasMeter},
};

use super::mode::{check::CheckTxMode, deliver::DeliverTxMode};

#[derive(Debug)]
pub struct ApplicationState<DB, AH: ABCIHandler> {
    pub(super) check_mode: CheckTxMode<DB, AH>,
    pub(super) deliver_mode: DeliverTxMode<DB, AH>,
    pub head_hash: [u8; 32],
    pub last_height: u32,
}

impl<DB: Database, AH: ABCIHandler> ApplicationState<DB, AH> {
    pub fn new(max_gas: Gas, multi_store: &ApplicationMultiBank<DB, AH::StoreKey>) -> Self {
        Self {
            check_mode: CheckTxMode::new(max_gas, multi_store.to_tx_kind()),
            deliver_mode: DeliverTxMode::new(max_gas, multi_store.to_tx_kind()),
            head_hash: multi_store.head_commit_hash(),
            last_height: multi_store.head_version(),
        }
    }

    pub fn replace_meter(&mut self, max_gas: Gas) {
        match max_gas {
            Gas::Infinite => {
                self.check_mode.block_gas_meter = GasMeter::new(Box::<InfiniteGasMeter>::default());
                self.deliver_mode.block_gas_meter =
                    GasMeter::new(Box::<InfiniteGasMeter>::default());
            }
            Gas::Finite(max_gas) => {
                self.check_mode.block_gas_meter =
                    GasMeter::new(Box::new(BasicGasMeter::new(max_gas)));
                self.deliver_mode.block_gas_meter =
                    GasMeter::new(Box::new(BasicGasMeter::new(max_gas)));
            }
        }
    }

    pub fn append_block_cache(&mut self, multi_store: &mut ApplicationMultiBank<DB, AH::StoreKey>) {
        self.check_mode.multi_store.append_block_cache(multi_store);
        self.deliver_mode
            .multi_store
            .append_block_cache(multi_store);
    }

    pub fn commit(&mut self, multi_store: &mut ApplicationMultiBank<DB, AH::StoreKey>) -> [u8; 32] {
        self.check_mode.multi_store.tx_cache_clear();
        self.check_mode.multi_store.block_cache_clear();

        self.deliver_mode.multi_store.tx_cache_clear();
        multi_store.consume_block_cache(&mut self.deliver_mode.multi_store);

        let hash = multi_store.commit();

        self.head_hash = hash;
        self.last_height = multi_store.head_version();

        hash
    }
}
