use std::collections::HashMap;
use std::collections::HashSet;

use gears::core::serializers::serialize_number_to_string;
use gears::params::parse_primitive_unwrap;
use gears::params::subspace;
use gears::params::subspace_mut;
use gears::params::Params;
use gears::params::ParamsDeserialize;
use gears::params::ParamsSubspaceKey;
use gears::store::types::prefix::immutable::ImmutablePrefixStore;
use gears::store::QueryableMultiKVStore;
use gears::store::ReadPrefixStore;
use gears::store::TransactionalMultiKVStore;
use gears::store::WritePrefixStore;
use gears::{
    store::{
        database::{prefix::PrefixDB, Database},
        StoreKey,
    },
    types::context::{QueryableContext, TransactionalContext},
};
use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_number_from_string;

const KEY_MAX_EXPECTED_TIME_PER_BLOCK: &str = "MaxExpectedTimePerBlock";

/// Params defines the set of Connection parameters.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct ConnectionParams {
    /// maximum expected time per block (in nanoseconds), used to enforce block delay. This parameter should reflect the
    /// largest amount of time that the chain might reasonably take to produce the next block under normal operating
    /// conditions. A safe choice is 3-5x the expected time per block.
    #[prost(uint64, tag = "1")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[serde(serialize_with = "serialize_number_to_string")]
    pub max_expected_time_per_block: u64,
}

impl Params for ConnectionParams {
    fn keys() -> HashSet<&'static str> {
        [KEY_MAX_EXPECTED_TIME_PER_BLOCK].into_iter().collect()
    }

    fn serialize(&self) -> HashMap<&'static str, Vec<u8>> {
        let mut hash_map = HashMap::with_capacity(1);

        hash_map.insert(
            KEY_MAX_EXPECTED_TIME_PER_BLOCK,
            format!("\"{}\"", self.max_expected_time_per_block).into_bytes(),
        );

        hash_map
    }
}

impl ParamsDeserialize for ConnectionParams {
    fn deserialize(mut fields: HashMap<&'static str, Vec<u8>>) -> Self {
        Self {
            max_expected_time_per_block: parse_primitive_unwrap(
                fields.remove(KEY_MAX_EXPECTED_TIME_PER_BLOCK),
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionParamsKeeper<SK, PSK> {
    pub store_key: SK,
    pub params_subspace_key: PSK,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> ConnectionParamsKeeper<SK, PSK> {
    pub fn _get<DB: Database, CTX: QueryableContext<DB, SK>>(&self, ctx: &CTX) -> ConnectionParams {
        let store = subspace(ctx, &self.store_key, &self.params_subspace_key);

        store.params().expect("required to exists")
    }

    pub fn set<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        params: ConnectionParams,
    ) {
        let mut store = subspace_mut(ctx, &self.store_key, &self.params_subspace_key);

        store.params_set(&params)
    }
}
