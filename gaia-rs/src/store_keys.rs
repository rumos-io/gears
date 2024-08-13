use gears::derive::{ParamsKeys, StoreKeys};
use strum::EnumIter;

#[derive(EnumIter, Debug, PartialEq, Eq, Hash, Clone, StoreKeys)]
#[skey(params = Params)]
pub enum GaiaStoreKey {
    #[skey(store_str = "bank")]
    Bank,
    #[skey(store_str = "acc")]
    Auth,
    #[skey(store_str = "params")]
    Params,
    #[skey(store_str = "staking")]
    Staking,
    #[skey(store_str = "ibc")]
    IBC,
    #[skey(store_str = "capability")]
    Capability,
}

#[derive(EnumIter, Debug, PartialEq, Eq, Hash, Clone, ParamsKeys)]
pub enum GaiaParamsStoreKey {
    #[pkey(prefix_str = "bank/")]
    Bank,
    #[pkey(prefix_str = "auth/")]
    Auth,
    #[pkey(prefix_str = "baseapp/")]
    BaseApp,
    #[pkey(prefix_str = "staking/")]
    Staking,
    #[pkey(prefix_str = "ibc/")]
    IBC,
    #[pkey(prefix_str = "capability/")]
    Capability,
}
