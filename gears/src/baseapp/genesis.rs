use crate::types::{address::AccAddress, base::coins::UnsignedCoins};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug, Clone, thiserror::Error)]
#[error("cannot add account at existing address {0}")]
pub struct GenesisError(pub AccAddress);

pub trait Genesis:
    std::fmt::Debug + Default + DeserializeOwned + Serialize + Clone + Send + Sync + 'static
{
    fn add_genesis_account(
        &mut self,
        address: AccAddress,
        coins: UnsignedCoins,
    ) -> Result<(), GenesisError>;
}
