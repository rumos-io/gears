use database::prefix::PrefixDB;
use kv_store::store::kv::{immutable::KVStore, mutable::KVStoreMut};
use tendermint::types::{chain_id::ChainId, proto::event::Event, time::timestamp::Timestamp};

use crate::types::store::kv::{mutable::StoreMut, Store};

pub mod block;
pub mod init;
pub mod query;
pub(crate) mod simple;
pub mod tx;

pub trait QueryableContext<DB, SK> {
    /// Fetches an immutable ref to a KVStore from the MultiStore.
    fn kv_store(&self, store_key: &SK) -> Store<'_, PrefixDB<DB>>;

    fn height(&self) -> u32;
    fn chain_id(&self) -> &ChainId;
}

pub trait InfallibleContext<DB, SK>: QueryableContext<DB, SK> {
    /// Fetches an immutable ref to a KVStore from the MultiStore.
    fn infallible_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>>;
}

pub trait TransactionalContext<DB, SK>: QueryableContext<DB, SK> {
    fn push_event(&mut self, event: Event);
    fn append_events(&mut self, events: Vec<Event>);
    fn events_drain(&mut self) -> Vec<Event>;

    /// Public interface for getting context timestamp. Default implementation returns `None`.
    fn get_time(&self) -> Timestamp;
    ///  Fetches an mutable ref to a KVStore from the MultiStore.
    fn kv_store_mut(&mut self, store_key: &SK) -> StoreMut<'_, PrefixDB<DB>>;
}

pub trait InfallibleContextMut<DB, SK>:
    TransactionalContext<DB, SK> + InfallibleContext<DB, SK>
{
    ///  Fetches an mutable ref to a KVStore from the MultiStore.
    fn infallible_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, PrefixDB<DB>>;
}
