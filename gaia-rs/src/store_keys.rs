use gears::derive::{ParamsKeys, StoreKeys};
use strum::EnumIter;

#[derive(EnumIter, Debug, PartialEq, Eq, Hash, Clone, StoreKeys)]
#[skey(params = Params)]
pub enum GaiaStoreKey {
    #[skey(to_string = "bank")]
    Bank,
    #[skey(to_string = "acc")]
    Auth,
    #[skey(to_string = "params")]
    Params,
    #[skey(to_string = "staking")]
    Staking,
    #[skey(to_string = "ibc")]
    IBC,
    #[skey(to_string = "capability")]
    Capability,
    #[skey(to_string = "gov")]
    Gov,
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
    #[pkey(to_string = "gov/")]
    Gov,
}
