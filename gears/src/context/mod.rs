use database::prefix::PrefixDB;
use kv_store::types::kv::{immutable::KVStore, mutable::KVStoreMut};
use tendermint::types::{proto::event::Event, time::Timestamp};

use crate::types::store::kv::{mutable::StoreMut, Store};

pub mod block;
pub mod init;
pub mod query;
pub(crate) mod simple;
pub mod tx;

pub trait QueryableContext<DB, SK> {
    fn height(&self) -> u64;
    // fn chain_id(&self) -> &ChainId;
}

pub trait ImmutableContext<DB, SK>: QueryableContext<DB, SK> + ImmutableGasContext<DB, SK> {
    /// Fetches an immutable ref to a KVStore from the MultiStore.
    fn infallible_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>>;
}

pub trait ImmutableGasContext<DB, SK>: QueryableContext<DB, SK> {
    /// Fetches an immutable ref to a KVStore from the MultiStore.
    fn kv_store(&self, store_key: &SK) -> Store<'_, PrefixDB<DB>>;
}

pub trait TransactionalContext<DB, SK>: QueryableContext<DB, SK> {
    fn push_event(&mut self, event: Event);
    fn append_events(&mut self, events: Vec<Event>);
    fn events_drain(&mut self) -> Vec<Event>;

    // TODO: change signature after changing struct `Header`
    /// Public interface for getting context timestamp. Default implementation returns `None`.
    fn get_time(&self) -> Option<Timestamp>;
}

pub trait MutableContext<DB, SK>:
    TransactionalContext<DB, SK> + ImmutableContext<DB, SK> + MutableGasContext<DB, SK>
{
    ///  Fetches an mutable ref to a KVStore from the MultiStore.
    fn infallible_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, PrefixDB<DB>>;
}

pub trait MutableGasContext<DB, SK>:
    TransactionalContext<DB, SK> + ImmutableGasContext<DB, SK>
{
    ///  Fetches an mutable ref to a KVStore from the MultiStore.
    fn kv_store_mut(&mut self, store_key: &SK) -> StoreMut<'_, PrefixDB<DB>>;
}
