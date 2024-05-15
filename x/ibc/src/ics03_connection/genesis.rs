//pub type GenesisState = ibc::core::connection::types::proto::v1::GenesisState;

use gears::core::serializers::serialize_number_to_string;
use ibc::core::connection::types::proto::v1::{ConnectionPaths, IdentifiedConnection};
use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_number_from_string;

use super::params::Params;

/// GenesisState defines the ibc connection submodule's genesis state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenesisState {
    pub connections: Vec<IdentifiedConnection>,
    pub client_connection_paths: Vec<ConnectionPaths>,
    /// the sequence for the next generated connection identifier
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[serde(serialize_with = "serialize_number_to_string")]
    pub next_connection_sequence: u64,
    pub params: Params,
}

impl Default for GenesisState {
    fn default() -> Self {
        Self {
            connections: vec![],
            client_connection_paths: vec![],
            next_connection_sequence: 0,
            params: Params {
                max_expected_time_per_block: 30_000_000_000,
            },
        }
    }
}
