use store_crate::database::Database;
use store_crate::{
    QueryableKVStore, QueryableMultiKVStore, TransactionalKVStore, TransactionalMultiKVStore,
};
use tendermint::types::{chain_id::ChainId, proto::event::Event};

pub mod gas;
pub mod init_context;
pub mod query_context;
pub mod tx;

pub trait QueryableContext<DB: Database, SK> {
    type KVStore: QueryableKVStore<DB>;
    type MultiStore: QueryableMultiKVStore<DB, SK>;

    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    fn kv_store(&self, store_key: &SK) -> &Self::KVStore; //AnyKVStore<'_, PrefixDB<DB>>;

    fn height(&self) -> u64;
    fn chain_id(&self) -> &ChainId;
    fn multi_store(&self) -> &Self::MultiStore;
}

pub trait TransactionalContext<DB: Database, SK>: QueryableContext<DB, SK> {
    type KVStoreMut: TransactionalKVStore<DB>;
    type MultiStoreMut: TransactionalMultiKVStore<DB, SK>;

    ///  Fetches an mutable ref to a KVStore from the MultiStore.
    fn kv_store_mut(&mut self, store_key: &SK) -> &mut Self::KVStoreMut; //AnyKVStore<'_, PrefixDB<DB>>;

    fn push_event(&mut self, event: Event);
    fn append_events(&mut self, events: Vec<Event>);
    fn events_drain(&mut self) -> Vec<Event>;
    fn multi_store_mut(&mut self) -> &mut Self::MultiStoreMut;
}
