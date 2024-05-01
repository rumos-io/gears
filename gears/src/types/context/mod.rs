use store_crate::types::kv::mutable::KVStoreMut;
use store_crate::types::kv::KVStore;
use store_crate::types::query::kv::QueryKVStore;
use store_crate::QueryableMultiKVStore;
use store_crate::{database::Database, TransactionalMultiKVStore};
use tendermint::types::{chain_id::ChainId, proto::event::Event};

pub mod init;
pub mod query;
pub mod tx;

pub trait Context<DB, SK> {
    fn height(&self) -> u64;
    fn chain_id(&self) -> &ChainId;
}

pub trait KVContext<DB, SK>: Context<DB, SK> {
    type MultiStore: QueryableMultiKVStore<DB, SK>;

    fn multi_store(&self) -> &Self::MultiStore;
    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    fn kv_store(&self, store_key: &SK) -> KVStore<'_, DB>;
}

pub trait QueryableContext<DB, SK>: Context<DB, SK> + KVContext<DB, SK> {
    ///  Fetches an immutable ref to a QueryKVStore from the MultiStore at specific height.
    fn query_store(&self, store_key: &SK) -> &QueryKVStore<DB>;
}

pub trait TransactionalContext<DB: Database, SK>: Context<DB, SK> + KVContext<DB, SK> {
    type MultiStoreMut: TransactionalMultiKVStore<DB, SK>;

    fn multi_store_mut(&mut self) -> &mut Self::MultiStoreMut;

    ///  Fetches an mutable ref to a KVStore from the MultiStore.
    fn kv_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, DB>;

    fn push_event(&mut self, event: Event);
    fn append_events(&mut self, events: Vec<Event>);
    fn events_drain(&mut self) -> Vec<Event>;
}
