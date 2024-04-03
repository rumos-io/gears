use database::{Database, PrefixDB};
use proto_messages::cosmos::tx::v1beta1::tx_metadata::{DenomUnit, Metadata};
use store_crate::{
    types::{kv::KVStore, multi::MultiStore},
    ReadMultiKVStore, StoreKey, WriteMultiKVStore,
};
use tendermint::informal::{abci::Event, block::Header, chain::Id};

use super::{Context, ContextMut};

pub struct TxContext<'a, DB, SK> {
    multi_store: &'a mut MultiStore<DB, SK>,
    pub height: u64,
    pub events: Vec<Event>,
    pub header: Header,
    _tx_bytes: Vec<u8>,
}

impl<'a, DB: Database, SK: StoreKey> TxContext<'a, DB, SK> {
    pub fn new(
        multi_store: &'a mut MultiStore<DB, SK>,
        height: u64,
        header: Header,
        tx_bytes: Vec<u8>,
    ) -> Self {
        TxContext {
            multi_store,
            height,
            events: vec![],
            header,
            _tx_bytes: tx_bytes,
        }
    }
}

impl<DB: Database, SK: StoreKey> Context<PrefixDB<DB>, SK> for TxContext<'_, DB, SK> {
    type KVStore = KVStore<PrefixDB<DB>>;

    fn kv_store(&self, store_key: &SK) -> &Self::KVStore {
        self.multi_store.kv_store(store_key)
    }

    fn height(&self) -> u64 {
        self.height
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

    fn chain_id(&self) -> &Id {
        todo!()
    }
}

impl<DB: Database, SK: StoreKey> ContextMut<PrefixDB<DB>, SK> for TxContext<'_, DB, SK> {
    type KVStoreMut = KVStore<PrefixDB<DB>>;

    fn kv_store_mut(&mut self, store_key: &SK) -> &mut Self::KVStoreMut {
        self.multi_store.kv_store_mut(store_key)
    }

    fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    fn append_events(&mut self, mut events: Vec<Event>) {
        self.events.append(&mut events);
    }
}
