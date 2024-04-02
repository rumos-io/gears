use crate::error::AppError;
use database::{Database, PrefixDB};
use proto_messages::cosmos::{
    ibc::types::core::host::identifiers::ChainId,
    tx::v1beta1::tx_metadata::{DenomUnit, Metadata},
};
use store_crate::{
    types::{
        multi::MultiStore,
        query::{kv::QueryKVStore, multi::QueryMultiStore},
    },
    ReadMultiKVStore, StoreKey,
};

use super::{Context, ReadContext};

pub struct QueryContext<'a, DB, SK> {
    pub multi_store: QueryMultiStore<'a, DB, SK>,
    pub height: u64,
    pub chain_id: ChainId,
}

impl<'a, DB: Database, SK: StoreKey> QueryContext<'a, DB, SK> {
    pub fn new(
        multi_store: &'a MultiStore<DB, SK>,
        version: u32,
        // chain_id: ChainId,
    ) -> Result<Self, AppError> {
        let multi_store = QueryMultiStore::new(multi_store, version)
            .map_err(|e| AppError::InvalidRequest(e.to_string()))?;
        Ok(QueryContext {
            multi_store,
            height: version as u64, // TODO:
            chain_id: ChainId::new("todo-900").expect("default should be valid"), // TODO:
        })
    }
}

impl<'a, SK: StoreKey, DB: Database> ReadContext<DB, SK> for QueryContext<'a, DB, SK> {
    type KVStore = QueryKVStore<'a, PrefixDB<DB>>;

    fn kv_store(&self, store_key: &SK) -> &Self::KVStore {
        self.multi_store.kv_store(store_key)
    }
}

impl<'a, DB, SK> Context<DB, SK> for QueryContext<'a, DB, SK> {
    fn height(&self) -> u64 {
        self.height
    }

    fn chain_id(&self) -> &tendermint::informal::chain::Id {
        // &self.chain_id
        unimplemented!() // TODO:NOW
    }

    fn metadata(&self) -> proto_messages::cosmos::tx::v1beta1::tx_metadata::Metadata {
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
