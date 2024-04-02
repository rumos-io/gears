use database::{Database, PrefixDB};
use proto_messages::cosmos::tx::v1beta1::tx_metadata::{DenomUnit, Metadata};
use store_crate::{
    types::{kv::KVStore, multi::MultiStore},
    ReadMultiKVStore, StoreKey, WriteMultiKVStore,
};
use tendermint::informal::{abci::Event, chain::Id};

use super::{Context, ContextMut, ReadContext, WriteContext};

#[derive(Debug)]
pub struct InitContext<'a, DB, SK> {
    multi_store: &'a mut MultiStore<DB, SK>,
    pub height: u64,
    pub events: Vec<Event>,
    pub chain_id: Id,
}

impl<'a, DB: Database, SK: StoreKey> InitContext<'a, DB, SK> {
    pub fn new(multi_store: &'a mut MultiStore<DB, SK>, height: u64, chain_id: Id) -> Self {
        InitContext {
            multi_store,
            height,
            events: vec![],
            chain_id,
        }
    }
}

impl<DB: Database, SK: StoreKey> Context<DB, SK> for InitContext<'_, DB, SK> {
    fn height(&self) -> u64 {
        self.height
    }

    fn chain_id(&self) -> &Id {
        &self.chain_id
    }

    fn metadata(&self) -> Metadata {
        Metadata {
            description: String::new(),
            denom_units: vec![
                DenomUnit {
                    denom: "ATOM".try_into().unwrap(),
                    exponent: 6,
                    aliases: Vec::new(),
                },
                DenomUnit {
                    denom: "uatom".try_into().unwrap(),
                    exponent: 0,
                    aliases: Vec::new(),
                },
            ],
            base: "uatom".into(),
            display: "ATOM".into(),
            name: String::new(),
            symbol: String::new(),
        }
    }
}

impl<DB: Database, SK: StoreKey> ContextMut<DB, SK> for InitContext<'_, DB, SK> {
    fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    fn append_events(&mut self, mut events: Vec<Event>) {
        self.events.append(&mut events);
    }
}

impl<DB: Database, SK: StoreKey> WriteContext<SK, DB> for InitContext<'_, DB, SK> {
    type KVStoreMut = KVStore<PrefixDB<DB>>;

    fn kv_store_mut(&mut self, store_key: &SK) -> &mut Self::KVStoreMut {
        self.multi_store.kv_store_mut(store_key)
    }
}

impl<SK: StoreKey, DB: Database> ReadContext<SK, DB> for InitContext<'_, DB, SK> {
    type KVStore = KVStore<PrefixDB<DB>>;

    fn kv_store(&self, store_key: &SK) -> &Self::KVStore {
        self.multi_store.kv_store(store_key)
    }
}
