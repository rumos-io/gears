use crate::types::context::init_context::InitContext;
use crate::types::context::tx_context::TxContext;
use database::{Database, PrefixDB};
use proto_messages::cosmos::tx::v1beta1::tx_metadata::Metadata;
use store_crate::{KVStore, StoreKey};
use tendermint::informal::abci::Event;

pub trait ContextTrait<T, SK> {
    fn height(&self) -> u64;
    fn chain_id(&self) -> &str;
    fn push_event(&mut self, event: Event);
    fn append_events(&mut self, events: Vec<Event>);
    fn metadata_get(&self) -> Metadata;
}

pub trait KVStoreRead<T, SK> {
    fn get_kv_store(&self, store_key: &SK) -> &KVStore<PrefixDB<T>>;
    fn get_mutable_kv_store(&mut self, store_key: &SK) -> &mut KVStore<PrefixDB<T>>;
}

pub trait DynamicContext<DB, SK>:
    ContextTrait<DB, SK> + KVStoreRead<DB, SK>
{
}

/// This is used when a method can be used in either a tx or init context
pub enum Context<'a, 'b, T: Database, SK: StoreKey> {
    TxContext(&'a mut TxContext<'b, T, SK>),
    InitContext(&'a mut InitContext<'b, T, SK>),
    DynamicContext(&'a mut dyn DynamicContext<T, SK>),
}

impl<DB: Database, SK: StoreKey> KVStoreRead<DB, SK> for Context<'_, '_, DB, SK> {
    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    fn get_kv_store(&self, store_key: &SK) -> &KVStore<PrefixDB<DB>> {
        match self {
            Context::TxContext(ctx) => ctx.get_kv_store(store_key),
            Context::InitContext(ctx) => ctx.get_kv_store(store_key),
            Context::DynamicContext(ctx) => ctx.get_kv_store(store_key),
        }
    }

    /// Fetches a mutable ref to a KVStore from the MultiStore.
    fn get_mutable_kv_store(&mut self, store_key: &SK) -> &mut KVStore<PrefixDB<DB>> {
        match self {
            Context::TxContext(ctx) => ctx.get_mutable_kv_store(store_key),
            Context::InitContext(ctx) => ctx.get_mutable_kv_store(store_key),
            Context::DynamicContext(ctx) => ctx.get_mutable_kv_store(store_key),
        }
    }
}

impl<T: Database, SK: StoreKey> Context<'_, '_, T, SK> {
    pub fn get_height(&self) -> u64 {
        match self {
            Context::TxContext(ctx) => ctx.height,
            Context::InitContext(ctx) => ctx.height,
            Context::DynamicContext(ctx) => ctx.height(),
        }
    }

    pub fn get_chain_id(&self) -> &str {
        match self {
            Context::TxContext(ctx) => ctx.header.chain_id.as_str(),
            Context::InitContext(ctx) => &ctx.chain_id,
            Context::DynamicContext(ctx) => ctx.chain_id(),
        }
    }

    pub fn push_event(&mut self, event: Event) {
        match self {
            Context::TxContext(ctx) => ctx.push_event(event),
            Context::InitContext(ctx) => ctx.events.push(event),
            Context::DynamicContext(ctx) => ctx.push_event(event),
        };
    }

    pub fn append_events(&mut self, events: impl IntoIterator<Item = Event>) {
        let mut events = events.into_iter().collect();
        match self {
            Context::TxContext(ctx) => ctx.append_events(events),
            Context::InitContext(ctx) => ctx.events.append(&mut events),
            Context::DynamicContext(ctx) => ctx.append_events(events),
        }
    }

    // Metdata is usualy consumed so we may clone
    pub fn metadata_get(&self) -> Metadata {
        match self {
            Context::TxContext(ctx) => ctx.metadata_get(),
            Context::InitContext(ctx) => ctx.metadata_get(),
            Context::DynamicContext(ctx) => ctx.metadata_get(),
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
