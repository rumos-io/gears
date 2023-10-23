use database::Database;
use tendermint_informal::{abci::Event, block::Header};

use store_crate::StoreKey;

use crate::types::gas::{gas_meter::GasMeter, infinite_meter::InfiniteGasMeter};

use super::context::{ContextTrait, EventManager, MS};

pub struct TxContext<T: Database, SK: StoreKey> {
    multi_store: MS<T, SK>,
    height: u64,
    pub events: Vec<Event>,
    header: Header,
    _tx_bytes: Vec<u8>,
    gas_meter: InfiniteGasMeter,       //TODO: Trait
    block_gas_meter: InfiniteGasMeter, //TODO: Trait
    event_manager: EventManager,       //TODO: Trait
}

impl<T: Database, SK: StoreKey> TxContext<T, SK> {
    pub fn new(multi_store: MS<T, SK>, height: u64, header: Header, tx_bytes: Vec<u8>) -> Self {
        TxContext {
            multi_store,
            height,
            events: vec![],
            header,
            _tx_bytes: tx_bytes,
            gas_meter: InfiniteGasMeter::new(),
            block_gas_meter: InfiniteGasMeter::new(),
            event_manager: EventManager,
        }
    }

    pub fn header_get(&self) -> &Header {
        &self.header
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

impl<'a, T: Database, SK: StoreKey> ContextTrait<T, SK> for TxContext<T, SK> {
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
