use database::prefix::PrefixDB;
use kv_store::types::kv::{immutable::KVStore, mutable::KVStoreMut};
use tendermint::types::{proto::event::Event, time::Timestamp};

pub mod block;
pub mod init;
pub mod query;
pub(crate) mod simple;
pub mod tx;

pub trait QueryableContext<DB, SK> {
    /// Fetches an immutable ref to a KVStore from the MultiStore.
    fn kv_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>>;

    fn height(&self) -> u64;
    // fn chain_id(&self) -> &ChainId;
}

pub trait TransactionalContext<DB, SK>: QueryableContext<DB, SK> {
    ///  Fetches an mutable ref to a KVStore from the MultiStore.
    fn kv_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, PrefixDB<DB>>;

    fn push_event(&mut self, event: Event);
    fn append_events(&mut self, events: Vec<Event>);
    fn events_drain(&mut self) -> Vec<Event>;

    // TODO: change signature after changing struct `Header`
    /// Public interface for getting context timestamp. Default implementation returns `None`.
    fn get_time(&self) -> Option<Timestamp>;
}
