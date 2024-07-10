use crate::{
    error::AppError,
    types::{address::AccAddress, base::coins::Coins},
};
use serde::{de::DeserializeOwned, Serialize};

pub trait Genesis:
    std::fmt::Debug + Default + DeserializeOwned + Serialize + Clone + Send + Sync + 'static
{
    fn add_genesis_account(&mut self, address: AccAddress, coins: Coins) -> Result<(), AppError>;
}
