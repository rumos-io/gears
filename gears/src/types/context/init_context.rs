use database::Database;
use tendermint_informal::abci::Event;

use store_crate::{MultiStore, StoreKey};

use crate::types::gas::{gas_meter::GasMeter, infinite_meter::InfiniteGasMeter};

use super::context::{ContextTrait, EventManager};

pub struct InitContext<'a, T: Database, SK: StoreKey> {
    pub multi_store: &'a mut MultiStore<T, SK>,
    height: u64,
    pub events: Vec<Event>,
    pub chain_id: String,
    gas_meter: InfiniteGasMeter,       //TODO: Trait
    block_gas_meter: InfiniteGasMeter, //TODO: Trait
    event_manager: EventManager,       //TODO: Trait
}

impl<'a, T: Database, SK: StoreKey> InitContext<'a, T, SK> {
    pub fn new(multi_store: &'a mut MultiStore<T, SK>, height: u64, chain_id: String) -> Self {
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

    pub fn multi_store(&self) -> &MultiStore<T, SK> {
        &self.multi_store
    }

    pub fn event_manager_set(&mut self, manager: EventManager) {
        self.event_manager = manager;
    }

    pub fn height(&self) -> u64 {
        self.height
    }
}

impl<'a, T: Database, SK: StoreKey> ContextTrait<T, SK> for InitContext<'a, T, SK> {
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

    fn multi_store_mut(&mut self) -> &mut MultiStore<T, SK> {
        &mut self.multi_store
    }
}
