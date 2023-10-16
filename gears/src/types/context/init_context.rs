use database::Database;
use tendermint_informal::abci::Event;

use store_crate::StoreKey;

use crate::types::gas::{gas_meter::GasMeter, infinite_meter::InfiniteGasMeter};

use super::{
    context::{Context, ContextTrait, EventManager, MS},
    tx_context::TxContext,
};

#[derive(Debug)]
pub struct InitContext<T: Database, SK: StoreKey> {
    pub multi_store: MS<T, SK>,
    height: u64,
    pub events: Vec<Event>,
    pub chain_id: String,
    gas_meter: InfiniteGasMeter,       //TODO: Trait
    block_gas_meter: InfiniteGasMeter, //TODO: Trait
    event_manager: EventManager,       //TODO: Trait
}

impl<'a, T: Database, SK: StoreKey> InitContext<T, SK> {
    pub fn new(multi_store: MS<T, SK>, height: u64, chain_id: String) -> Self {
        InitContext {
            multi_store,
            height,
            events: vec![],
            chain_id,
            gas_meter: InfiniteGasMeter::new(),
            block_gas_meter: InfiniteGasMeter::new(),
            event_manager: EventManager,
        }
    }

    pub fn multi_store(&self) -> &MS<T, SK> {
        &self.multi_store
    }

    pub fn event_manager_set(&mut self, manager: EventManager) {
        self.event_manager = manager;
    }

    pub fn height(&self) -> u64 {
        self.height
    }
}

impl<'a, T: Database, SK: StoreKey> ContextTrait<T, SK> for InitContext<T, SK> {
    fn gas_meter(&self) -> &dyn GasMeter {
        &self.gas_meter
    }

    fn block_gas_meter(&self) -> &dyn GasMeter {
        &self.block_gas_meter
    }

    fn gas_meter_mut(&mut self) -> &mut dyn GasMeter {
        &mut self.gas_meter
    }

    fn block_gas_meter_mut(&mut self) -> &mut dyn GasMeter {
        &mut self.block_gas_meter
    }

    fn get_height(&self) -> u64 {
        self.height
    }

    fn push_event(&mut self, event: Event) {
        self.events.push(event)
    }

    fn append_events(&mut self, mut events: Vec<Event>) {
        self.events.append(&mut events)
    }
}

impl<T: Database, SK: StoreKey> TryFrom<Context<T, SK>> for InitContext<T, SK> {
    type Error = TxContext<T, SK>;

    fn try_from(value: Context<T, SK>) -> Result<Self, Self::Error> {
        match value {
            Context::TxContext(var) => Err(var),
            Context::InitContext(var) => Ok(var),
        }
    }
}
