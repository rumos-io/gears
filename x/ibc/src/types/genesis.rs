use serde::{Deserialize, Serialize};

use crate::ics02_client;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GenesisState {
    pub client_genesis: ics02_client::GenesisState,
}

// impl Default for GenesisState {
//     fn default() -> Self {

//         Self {}
//     }
// }
