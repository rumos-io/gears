use std::sync::{Arc, RwLock};

use database::{Database, PrefixDB};
use ibc_relayer::util::lock::LockExt;
use tendermint_informal::{abci::Event, block::Header};

use store_crate::{KVStore, MultiStore, QueryKVStore, QueryMultiStore, StoreKey};

use crate::error::AppError;

use super::gas::{gas_meter::GasMeter, infinite_meter::InfiniteGasMeter};

/// Execution mode of transaction
#[derive(Debug, PartialEq)]
pub enum ExecMode {
    /// Check a transaction
    ExecModeCheck,
    /// Recheck a (pending) transaction after a commit
    ExecModeReCheck,
    /// Simulate a transaction
    ExecModeSimulate,
    /// Prepare a block proposal
    ExecModePrepareProposal,
    /// Process a block proposal
    ExecModeProcessProposal,
    /// Extend or verify a pre-commit vote
    ExecModeVoteExtension,
    /// Finalize a block proposal
    ExecModeFinalize,
}

pub type MS<T: Database, SK: StoreKey> = Arc<RwLock<MultiStore<T, SK>>>;

pub trait ContextTrait<T: Database, SK: StoreKey> {
    fn gas_meter(&self) -> &dyn GasMeter;
    fn block_gas_meter(&self) -> &dyn GasMeter;
    fn gas_meter_mut(&mut self) -> &mut dyn GasMeter;
    fn block_gas_meter_mut(&mut self) -> &mut dyn GasMeter;
    // fn as_any<'b>(&'b mut self) -> Context<T, SK>; //TODO:
    // ///  Fetches an immutable ref to a KVStore from the MultiStore.
    // fn get_kv_store(&self, store_key: &SK) -> &KVStore<PrefixDB<T>>;
    // /// Fetches a mutable ref to a KVStore from the MultiStore.
    // fn get_mutable_kv_store(&mut self, store_key: &SK) -> &mut KVStore<PrefixDB<T>>;
    fn get_height(&self) -> u64;
    fn push_event(&mut self, event: Event);
    fn append_events(&mut self, events: Vec<Event>);
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

    // fn get_kv_store(&self, store_key: &SK) -> &KVStore<PrefixDB<T>> {
    //     self.multi_store.acquire_write().get_kv_store(store_key)
    // }

    // fn get_mutable_kv_store(&mut self, store_key: &SK) -> &mut KVStore<PrefixDB<T>> {
    //     self.multi_store.acquire_write().get_mutable_kv_store(store_key)
    // }

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

pub struct TxContext<T: Database, SK: StoreKey> {
    multi_store: MS<T, SK>,
    height: u64,
    pub events: Vec<Event>,
    header: Header,
    _tx_bytes: Vec<u8>,
    gas_meter: InfiniteGasMeter,       //TODO: Trait
    block_gas_meter: InfiniteGasMeter, //TODO: Trait
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
        }
    }

    fn get_header(&self) -> &Header {
        &self.header
    }

    fn multi_store(&self) -> &MS<T, SK> {
        &self.multi_store
    }
}

pub struct InitContext<T: Database, SK: StoreKey> {
    pub multi_store: MS<T, SK>,
    height: u64,
    pub events: Vec<Event>,
    pub chain_id: String,
    gas_meter: InfiniteGasMeter,       //TODO: Trait
    block_gas_meter: InfiniteGasMeter, //TODO: Trait
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
        }
    }

    pub fn multi_store(&self) -> &MS<T, SK> {
        &self.multi_store
    }

    // fn as_any(&self) -> Context<T, SK> {
    //     Context::InitContext(self)
    // }
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

    // fn get_kv_store(&self, store_key: &SK) -> &KVStore<PrefixDB<T>> {
    //     self.multi_store.acquire_write().get_kv_store(store_key)
    // }

    // fn get_mutable_kv_store(&mut self, store_key: &SK) -> &mut KVStore<PrefixDB<T>> {
    //     self.multi_store.acquire_write().get_mutable_kv_store(store_key)
    // }

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

/// This is used when a method can be used in either a tx or init context
pub enum Context<'a, T: Database, SK: StoreKey> {
    TxContext(&'a mut TxContext<T, SK>),
    InitContext(&'a mut InitContext<T, SK>),
}

impl<'a, T: Database, SK: StoreKey> Context<'a, T, SK> {
    // ///  Fetches an immutable ref to a KVStore from the MultiStore.
    // pub fn get_kv_store(&self, store_key: &SK) -> &KVStore<PrefixDB<T>> {
    //     match self {
    //         Context::TxContext(ctx) => return ctx.get_kv_store(store_key),
    //         Context::InitContext(ctx) => return ctx.multi_store.acquire_read().get_kv_store(store_key),
    //     }
    // }

    // /// Fetches a mutable ref to a KVStore from the MultiStore.
    // pub fn get_mutable_kv_store(&mut self, store_key: &SK) -> &mut KVStore<PrefixDB<T>> {
    //     match self {
    //         Context::TxContext(ctx) => return ctx.get_mutable_kv_store(store_key),
    //         Context::InitContext(ctx) => return ctx.multi_store.acquire_write().get_mutable_kv_store(store_key),
    //     }
    // }

    pub fn get_height(&self) -> u64 {
        match self {
            Context::TxContext(ctx) => ctx.height,
            Context::InitContext(ctx) => ctx.height,
        }
    }

    pub fn get_chain_id(&self) -> &str {
        match self {
            Context::TxContext(ctx) => ctx.header.chain_id.as_str(),
            Context::InitContext(ctx) => &ctx.chain_id,
        }
    }

    pub fn push_event(&mut self, event: Event) {
        match self {
            Context::TxContext(ctx) => ctx.push_event(event),
            Context::InitContext(ctx) => ctx.events.push(event),
        };
    }

    pub fn append_events(&mut self, mut events: Vec<Event>) {
        match self {
            Context::TxContext(ctx) => ctx.append_events(events),
            Context::InitContext(ctx) => ctx.events.append(&mut events),
        }
    }

    pub fn gas_meter(&self) -> &dyn GasMeter {
        match self {
            Context::TxContext(ctx) => ctx.gas_meter(),
            Context::InitContext(ctx) => ctx.gas_meter(),
        }
    }

    pub fn block_gas_meter(&self) -> &dyn GasMeter {
        match self {
            Context::TxContext(ctx) => ctx.block_gas_meter(),
            Context::InitContext(ctx) => ctx.block_gas_meter(),
        }
    }

    pub fn multi_store(&self) -> &MS<T, SK> {
        match self {
            Context::TxContext(ctx) => ctx.multi_store(),
            Context::InitContext(ctx) => ctx.multi_store(),
        }
    }

    pub fn with_multi_store(&self) -> Self {
        unimplemented!() //TODO
    }
}

impl<'a, T: Database, SK: StoreKey> From<&'a mut TxContext<T, SK>> for Context<'a, T, SK> {
    fn from(value: &'a mut TxContext<T, SK>) -> Self {
        Self::TxContext(value)
    }
}

impl<'a, T: Database, SK: StoreKey> From<&'a mut InitContext<T, SK>> for Context<'a, T, SK> {
    fn from(value: &'a mut InitContext<T, SK>) -> Self {
        Self::InitContext(value)
    }
}

pub struct QueryContext<'a, T: Database, SK: StoreKey> {
    pub multi_store: QueryMultiStore<'a, T, SK>,
    //_height: u64,
}

impl<'a, T: Database, SK: StoreKey> QueryContext<'a, T, SK> {
    pub fn new(multi_store: &'a MultiStore<T, SK>, version: u32) -> Result<Self, AppError> {
        let multi_store = QueryMultiStore::new(multi_store, version)
            .map_err(|e| AppError::InvalidRequest(e.to_string()))?;
        Ok(QueryContext {
            multi_store,
            //_height: height,
        })
    }

    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    pub fn get_kv_store(&self, store_key: &SK) -> &QueryKVStore<PrefixDB<T>> {
        return self.multi_store.get_kv_store(store_key);
    }

    // pub fn _get_height(&self) -> u64 {
    //     self._height
    // }
}

// type Context struct {
// 	ctx           context.Context
// 	ms            MultiStore
// 	header        tmproto.Header
// 	headerHash    tmbytes.HexBytes
// 	chainID       string
// 	txBytes       []byte
// 	logger        log.Logger
// 	voteInfo      []abci.VoteInfo
// 	gasMeter      GasMeter
// 	blockGasMeter GasMeter
// 	checkTx       bool
// 	recheckTx     bool // if recheckTx == true, then checkTx must also be true
// 	minGasPrice   DecCoins
// 	consParams    *abci.ConsensusParams
// 	eventManager  *EventManager
// }
