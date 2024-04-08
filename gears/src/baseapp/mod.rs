mod baseapp;
mod params;
pub mod run;

pub use baseapp::*;

use ibc_proto::address::AccAddress;
use serde::{de::DeserializeOwned, Serialize};

use crate::{error::AppError, types::base::send::SendCoins};

pub trait Genesis: Default + DeserializeOwned + Serialize + Clone + Send + Sync + 'static {
    fn add_genesis_account(
        &mut self,
        address: AccAddress,
        coins: SendCoins,
    ) -> Result<(), AppError>;
}
