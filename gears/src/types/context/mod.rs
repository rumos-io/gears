pub mod tx;
use store_crate::database::Database;
use store_crate::{QueryableKVStore, TransactionalKVStore};
use tendermint::types::{chain_id::ChainId, proto::event::Event};

use super::gas::gas_meter::Gas;

pub mod gas;
pub mod init_context;
pub mod query_context;
pub mod tx_context;

pub trait QueryableContext<DB: Database, SK> {
    type KVStore: QueryableKVStore<DB>;

    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    fn kv_store(&self, store_key: &SK) -> &Self::KVStore; //AnyKVStore<'_, PrefixDB<DB>>;

    fn height(&self) -> u64;
    fn chain_id(&self) -> &ChainId;
}

pub trait TransactionalContext<DB: Database, SK>: QueryableContext<DB, SK> {
    type KVStoreMut: TransactionalKVStore<DB>;

    ///  Fetches an mutable ref to a KVStore from the MultiStore.
    fn kv_store_mut(&mut self, store_key: &SK) -> &mut Self::KVStoreMut; //AnyKVStore<'_, PrefixDB<DB>>;

    fn push_event(&mut self, event: Event);
    fn append_events(&mut self, events: Vec<Event>);
}

/// Execution mode of transaction
#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, Clone, Default)]
pub struct ContextOptions {
    pub max_gas: Gas,
}
