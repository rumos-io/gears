mod baseapp;
mod params;
pub mod run;

pub use baseapp::*;

use crate::{error::AppError, types::base::send::SendCoins};
use ibc_types::address::AccAddress;
use serde::{de::DeserializeOwned, Serialize};

pub trait Genesis: Default + DeserializeOwned + Serialize + Clone + Send + Sync + 'static {
    fn add_genesis_account(
        &mut self,
        address: AccAddress,
        coins: SendCoins,
    ) -> Result<(), AppError>;
}
