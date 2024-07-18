use crate::types::{address::AccAddress, base::coins::UnsignedCoins};
use serde::{de::DeserializeOwned, Serialize};

pub trait Genesis:
    std::fmt::Debug + Default + DeserializeOwned + Serialize + Clone + Send + Sync + 'static
{
    fn add_genesis_account(
        &mut self,
        address: AccAddress,
        coins: UnsignedCoins,
    ) -> anyhow::Result<()>;
}
