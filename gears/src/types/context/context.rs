use crate::types::context::init_context::InitContext;
use crate::types::context::tx_context::TxContext;
use database::{Database, PrefixDB};
use store_crate::{KVStore, StoreKey};
use tendermint_informal::abci::Event;

/// This is used when a method can be used in either a tx or init context
pub enum Context<'a, 'b, T: Database, SK: StoreKey> {
    TxContext(&'a mut TxContext<'b, T, SK>),
    InitContext(&'a mut InitContext<'b, T, SK>),
}

impl<'a, 'b, T: Database, SK: StoreKey> Context<'a, 'b, T, SK> {
    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    pub fn get_kv_store(&self, store_key: &SK) -> &KVStore<PrefixDB<T>> {
        match self {
            Context::TxContext(ctx) => ctx.get_kv_store(store_key),
            Context::InitContext(ctx) => ctx.multi_store.get_kv_store(store_key),
        }
    }

    /// Fetches a mutable ref to a KVStore from the MultiStore.
    pub fn get_mutable_kv_store(&mut self, store_key: &SK) -> &mut KVStore<PrefixDB<T>> {
        match self {
            Context::TxContext(ctx) => ctx.get_mutable_kv_store(store_key),
            Context::InitContext(ctx) => ctx.multi_store.get_mutable_kv_store(store_key),
        }
    }

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
}

impl<'a, 'b, T: Database, SK: StoreKey> From<&'a mut TxContext<'b, T, SK>>
    for Context<'a, 'b, T, SK>
{
    fn from(value: &'a mut TxContext<'b, T, SK>) -> Self {
        Self::TxContext(value)
    }
}

impl<'a, 'b, T: Database, SK: StoreKey> From<&'a mut InitContext<'b, T, SK>>
    for Context<'a, 'b, T, SK>
{
    fn from(value: &'a mut InitContext<'b, T, SK>) -> Self {
        Self::InitContext(value)
    }
}

/// Execution mode of transaction
#[derive(Debug, PartialEq)]
pub enum ExecMode {
    /// Check a transaction
    Check,
    /// Recheck a (pending) transaction after a commit
    ReCheck,
    /// Simulate a transaction
    Simulate,
    /// Prepare a block proposal
    PrepareProposal,
    /// Process a block proposal
    ProcessProposal,
    /// Extend or verify a pre-commit vote
    VoteExtension,
    /// Finalize a block proposal
    Finalize,
    /// Deliver a transaction
    Deliver,
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
