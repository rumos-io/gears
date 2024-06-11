use gears::{params::ParamsSubspaceKey, store::StoreKey};
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

#[derive(EnumIter, Debug, PartialEq, Eq, Hash, Clone)]
pub enum GaiaParamsStoreKey {
    Bank,
    Auth,
    BaseApp,
    Staking,
    IBC,
    Capability,
}

/// WARNING: a key name must not be a prefix of another, there is currently
/// no check in the SDK to prevent this.
impl ParamsSubspaceKey for GaiaParamsStoreKey {
    fn name(&self) -> &'static str {
        match self {
            Self::Bank => "bank/",
            Self::Auth => "auth/",
            Self::BaseApp => "baseapp/",
            Self::Staking => "staking/",
            Self::IBC => "ibc/",
            Self::Capability => "capability/",
        }
    }
}
