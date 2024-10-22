use crate::types::{address::AccAddress, base::coins::UnsignedCoins};
use serde::{de::DeserializeOwned, Serialize};

pub use null_genesis::NullGenesis;

#[derive(Debug, Clone, thiserror::Error)]
#[error("cannot add account at existing address {0}")]
pub struct GenesisError(pub AccAddress);

pub trait Genesis:
    std::fmt::Debug + Default + DeserializeOwned + Serialize + Clone + Send + Sync + 'static
{
    // TODO: REMOVE?
    fn add_genesis_account(
        &mut self,
        address: AccAddress,
        coins: UnsignedCoins,
    ) -> Result<(), GenesisError>;
}

mod null_genesis {
    use super::*;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
    pub struct NullGenesis();

    impl Genesis for NullGenesis {
        fn add_genesis_account(
            &mut self,
            _: AccAddress,
            _: UnsignedCoins,
        ) -> Result<(), GenesisError> {
            Ok(())
        }
    }
}
