use database::{Database, PrefixDB};
use store_crate::types::kv::mutable::KVStoreMut;
use store_crate::types::kv::KVStore;
use store_crate::types::multi::mutable::MultiStoreMut;
use store_crate::types::multi::MultiStore;
use tendermint::types::{chain_id::ChainId, proto::event::Event};

pub mod init;
pub mod query;
pub mod tx;

pub trait QueryableContext<DB, SK> {
    fn multi_store(&self) -> MultiStore<'_, DB, SK>;
    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    fn kv_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>>;

    fn height(&self) -> u64;
    fn chain_id(&self) -> &ChainId;
}

pub trait TransactionalContext<DB: Database, SK>: QueryableContext<DB, SK> {
    fn multi_store_mut(&mut self) -> MultiStoreMut<'_, DB, SK>;

    ///  Fetches an mutable ref to a KVStore from the MultiStore.
    fn kv_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, PrefixDB<DB>>;

    fn push_event(&mut self, event: Event);
    fn append_events(&mut self, events: Vec<Event>);
    fn events_drain(&mut self) -> Vec<Event>;
}
