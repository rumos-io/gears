use serde::{Deserialize, Serialize};

use crate::{ics02_client, ics03_connection, ics04_channel};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GenesisState {
    pub client_genesis: ics02_client::GenesisState,
    pub connection_genesis: ics03_connection::GenesisState,
    pub channel_genesis: ics04_channel::GenesisState,
}

// impl Default for GenesisState {
//     fn default() -> Self {

//         Self {}
//     }
// }
