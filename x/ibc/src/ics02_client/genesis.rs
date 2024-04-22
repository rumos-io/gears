//pub type GenesisState = ibc::core::client::types::proto::v1::GenesisState;

use ibc::core::client::types::proto::v1::{
    ClientConsensusStates, IdentifiedClientState, IdentifiedGenesisMetadata,
};
use serde::{Deserialize, Serialize};

use super::params::Params;

/// GenesisState defines the ibc client submodule's genesis state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenesisState {
    /// client states with their corresponding identifiers
    pub clients: Vec<IdentifiedClientState>,
    /// consensus states from each client
    pub clients_consensus: Vec<ClientConsensusStates>,
    /// metadata from each client
    pub clients_metadata: Vec<IdentifiedGenesisMetadata>,

    pub params: Params,
    /// Deprecated: create_localhost has been deprecated.
    /// The localhost client is automatically created at genesis.
    pub create_localhost: bool,
    /// the sequence for the next generated client identifier
    pub next_client_sequence: u64,
}

impl Default for GenesisState {
    fn default() -> Self {
        Self {
            clients: vec![],
            clients_consensus: vec![],
            clients_metadata: vec![],
            params: Params {
                allowed_clients: vec!["06-solomachine".into(), "07-tendermint".into()],
            },
            create_localhost: false,
            next_client_sequence: 0,
        }
    }
}
