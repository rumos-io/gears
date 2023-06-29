use gears::x::auth::Params;
use proto_messages::cosmos::auth::v1beta1::BaseAccount;
use serde::{Deserialize, Serialize};

//use crate::Params;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenesisState {
    pub accounts: Vec<BaseAccount>,
    pub params: Params,
}
