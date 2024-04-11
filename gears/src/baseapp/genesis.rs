use crate::{error::AppError, types::base::send::SendCoins};
use core_types::address::AccAddress;
use serde::{de::DeserializeOwned, Serialize};

pub trait Genesis: Default + DeserializeOwned + Serialize + Clone + Send + Sync + 'static {
    fn add_genesis_account(
        &mut self,
        address: AccAddress,
        coins: SendCoins,
    ) -> Result<(), AppError>;
}
