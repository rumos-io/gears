use database::{Database, PrefixDB};
use proto_messages::cosmos::tx::v1beta1::tx_metadata::Metadata;
use store_crate::{ReadKVStore, WriteKVStore};
use tendermint::informal::{abci::Event, chain::Id};

pub mod init_context;
pub mod query_context;
pub mod tx_context;

pub trait Context<DB, SK> {
    fn height(&self) -> u64;
    fn chain_id(&self) -> &Id;
    fn metadata(&self) -> Metadata;
}

pub trait ContextMut<DB, SK>: Context<DB, SK> {
    fn push_event(&mut self, event: Event);
    fn append_events(&mut self, events: Vec<Event>);
}

pub trait ReadContext<DB: Database, SK> {
    type KVStore: ReadKVStore<PrefixDB<DB>>;

    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    fn kv_store(&self, store_key: &SK) -> &Self::KVStore; //AnyKVStore<'_, PrefixDB<DB>>;
}

pub trait WriteContext<DB: Database, SK>: ReadContext<DB, SK> {
    type KVStoreMut: WriteKVStore<PrefixDB<DB>>;

    ///  Fetches an mutable ref to a KVStore from the MultiStore.
    fn kv_store_mut(&mut self, store_key: &SK) -> &mut Self::KVStoreMut; //AnyKVStore<'_, PrefixDB<DB>>;
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
