use gears::{derive::ParamsKeys, store::StoreKey};
use strum::EnumIter;

#[derive(EnumIter, Debug, PartialEq, Eq, Hash, Clone)]
pub enum GaiaStoreKey {
    Bank,
    Auth,
    Params,
    Staking,
    IBC,
    Capability,
}

/// WARNING: a key name must not be a prefix of another, there is currently
/// no check in the SDK to prevent this.
impl StoreKey for GaiaStoreKey {
    fn name(&self) -> &'static str {
        match self {
            GaiaStoreKey::Bank => "bank",
            GaiaStoreKey::Auth => "acc",
            GaiaStoreKey::Params => "params",
            GaiaStoreKey::Staking => "staking",
            GaiaStoreKey::IBC => "ibc",
            GaiaStoreKey::Capability => "capability",
        }
    }

    fn params() -> &'static Self {
        const PARAM_KEY: GaiaStoreKey = GaiaStoreKey::Params;

        &PARAM_KEY
    }
}

#[derive(EnumIter, Debug, PartialEq, Eq, Hash, Clone, ParamsKeys)]
pub enum GaiaParamsStoreKey {
    #[pkey(to_string = "bank/")]
    Bank,
    #[pkey(to_string = "auth/")]
    Auth,
    #[pkey(to_string = "baseapp/")]
    BaseApp,
    #[pkey(to_string = "staking/")]
    Staking,
    #[pkey(to_string = "ibc/")]
    IBC,
    #[pkey(to_string = "capability/")]
    Capability,
}
