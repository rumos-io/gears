use gears::core::serializers::serialize_number_to_string;
use gears::store::types::prefix::immutable::ImmutablePrefixStore;
use gears::store::QueryableMultiKVStore;
use gears::store::ReadPrefixStore;
use gears::store::TransactionalMultiKVStore;
use gears::store::WritePrefixStore;
use gears::{
    params::ParamsSubspaceKey,
    store::{
        database::{Database, PrefixDB},
        StoreKey,
    },
    types::context::{QueryableContext, TransactionalContext},
};
use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_number_from_string;

const KEY_MAX_EXPECTED_TIME_PER_BLOCK: &[u8; 23] = b"MaxExpectedTimePerBlock";

/// Params defines the set of Connection parameters.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct Params {
    /// maximum expected time per block (in nanoseconds), used to enforce block delay. This parameter should reflect the
    /// largest amount of time that the chain might reasonably take to produce the next block under normal operating
    /// conditions. A safe choice is 3-5x the expected time per block.
    #[prost(uint64, tag = "1")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[serde(serialize_with = "serialize_number_to_string")]
    pub max_expected_time_per_block: u64,
}

#[derive(Debug, Clone)]
pub struct ConnectionParamsKeeper<SK, PSK> {
    pub params_keeper: gears::params::Keeper<SK, PSK>,
    pub params_subspace_key: PSK,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> ConnectionParamsKeeper<SK, PSK> {
    fn _get_raw_param<DB: Database>(key: &[u8], store: &ImmutablePrefixStore<'_, DB>) -> Vec<u8> {
        store
            .get(key)
            .expect("key should be set in kv store")
            .clone()
    }

    fn _parse_param(value: Vec<u8>) -> u64 {
        String::from_utf8(value)
            .expect("should be valid utf-8")
            .strip_suffix('\"')
            .expect("should have suffix")
            .strip_prefix('\"')
            .expect("should have prefix")
            .parse()
            .expect("should be valid u64")
    }

    pub fn _get<DB: Database, KV: QueryableMultiKVStore<DB, SK>>(&self, ctx: &KV) -> Params {
        let store = self
            .params_keeper
            .raw_subspace(ctx, &self.params_subspace_key);

        let raw = Self::_get_raw_param::<DB>(KEY_MAX_EXPECTED_TIME_PER_BLOCK, &store);
        let max_expected_time_per_block = Self::_parse_param(raw);

        Params {
            max_expected_time_per_block,
        }
    }

    pub fn set<DB: Database, KV: TransactionalMultiKVStore<DB, SK>>(
        &self,
        ctx: &mut KV,
        params: Params,
    ) {
        let mut store = self
            .params_keeper
            .raw_subspace_mut(ctx, &self.params_subspace_key);

        store.set(
            KEY_MAX_EXPECTED_TIME_PER_BLOCK.to_owned(),
            format!("\"{}\"", params.max_expected_time_per_block).into_bytes(),
        );
    }
}
