use std::collections::HashMap;
use std::collections::HashSet;

use gears::context::InfallibleContext;
use gears::context::InfallibleContextMut;
use gears::core::serializers::serialize_number_to_string;
use gears::extensions::corruption::UnwrapCorrupt;
use gears::params::infallible_subspace;
use gears::params::infallible_subspace_mut;
use gears::params::ParamKind;
use gears::params::ParamsDeserialize;
use gears::params::ParamsSerialize;
use gears::params::ParamsSubspaceKey;
use gears::store::store::prefix::immutable::ImmutablePrefixStore;
use gears::{
    context::{QueryableContext, TransactionalContext},
    store::{
        database::{prefix::PrefixDB, Database},
        StoreKey,
    },
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

impl ParamsSerialize for ConnectionParams {
    fn keys() -> HashSet<&'static str> {
        [KEY_MAX_EXPECTED_TIME_PER_BLOCK].into_iter().collect()
    }

    fn to_raw(&self) -> Vec<(&'static str, Vec<u8>)> {
        let mut hash_map = Vec::with_capacity(1);

        hash_map.push((
            KEY_MAX_EXPECTED_TIME_PER_BLOCK,
            format!("\"{}\"", self.max_expected_time_per_block).into_bytes(),
        ));

        hash_map
    }
}

impl ParamsDeserialize for ConnectionParams {
    fn from_raw(mut fields: HashMap<&'static str, Vec<u8>>) -> Self {
        Self {
            max_expected_time_per_block: ParamKind::U64
                .parse_param(
                    fields
                        .remove(KEY_MAX_EXPECTED_TIME_PER_BLOCK)
                        .unwrap_or_corrupt(),
                )
                .unsigned_64()
                .unwrap_or_corrupt(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionParamsKeeper<PSK> {
    pub params_subspace_key: PSK,
}

impl<PSK: ParamsSubspaceKey> ConnectionParamsKeeper<PSK> {
    pub fn _get<DB: Database, SK: StoreKey, CTX: InfallibleContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> ConnectionParams {
        let store = infallible_subspace(ctx, &self.params_subspace_key);

        store.params().unwrap_or_default()
    }

    pub fn set<DB: Database, SK: StoreKey, CTX: InfallibleContextMut<DB, SK>>(
        &self,
        ctx: &mut CTX,
        params: ConnectionParams,
    ) {
        let mut store = infallible_subspace_mut(ctx, &self.params_subspace_key);

        store.params_set(&params)
    }
}
