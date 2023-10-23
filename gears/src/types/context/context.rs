use std::sync::{Arc, RwLock};

use database::Database;
use tendermint_informal::abci::Event;

use store_crate::{StoreKey, MultiStore};

use crate::types::gas::gas_meter::GasMeter;

use super::{init_context::InitContext, tx_context::TxContext};

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

pub struct EventManager; //TODO: Replace with implementation

pub type MS<T, SK> = Arc<RwLock<MultiStore<T, SK>>>;

pub trait ContextTrait<T: Database, SK: StoreKey> {
    fn gas_meter(&self) -> &dyn GasMeter;
    fn block_gas_meter(&self) -> &dyn GasMeter;
    fn gas_meter_mut(&mut self) -> &mut dyn GasMeter;
    fn block_gas_meter_mut(&mut self) -> &mut dyn GasMeter;
    fn get_height(&self) -> u64;
    fn push_event(&mut self, event: Event);
    fn append_events(&mut self, events: Vec<Event>);
}

/// This is used when a method can be used in either a tx or init context
pub enum Context<'a, T: Database, SK: StoreKey> {
    TxContext(&'a mut TxContext<T, SK>),
    InitContext(&'a mut InitContext<T, SK>),
}

impl<'a, T: Database, SK: StoreKey> Context<'a, T, SK> {
    pub fn get_height(&self) -> u64 {
        match self {
            Context::TxContext(ctx) => ctx.height(),
            Context::InitContext(ctx) => ctx.height(),
        }
    }

    pub fn get_chain_id(&self) -> &str {
        match self {
            Context::TxContext(ctx) => ctx.header_get().chain_id.as_str(),
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

    pub fn event_manager_set(&mut self, manager: EventManager) {
        match self {
            Context::TxContext(ctx) => ctx.event_manager_set(manager),
            Context::InitContext(ctx) => ctx.event_manager_set(manager),
        }
    }

    pub fn gas_meter_mut(&mut self) -> &mut dyn GasMeter {
        match self {
            Context::TxContext(ctx) => ctx.gas_meter_mut(),
            Context::InitContext(ctx) => ctx.gas_meter_mut(),
        }
    }

    pub fn block_gas_meter_mut(&mut self) -> &mut dyn GasMeter {
        match self {
            Context::TxContext(ctx) => ctx.block_gas_meter_mut(),
            Context::InitContext(ctx) => ctx.block_gas_meter_mut(),
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
