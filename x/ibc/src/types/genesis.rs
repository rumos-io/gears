use serde::{Deserialize, Serialize};

use crate::{ics02_client, ics03_connection};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GenesisState {
    pub client_genesis: ics02_client::GenesisState,
    pub connection_genesis: ics03_connection::GenesisState,
}

// impl Default for GenesisState {
//     fn default() -> Self {

//         Self {}
//     }
// }
