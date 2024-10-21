use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use gears::{
    application::keepers::params::ParamsKeeper,
    params::{ParamsDeserialize, ParamsSerialize, ParamsSubspaceKey},
    types::{decimal256::Decimal256, denom::Denom},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintingParams {
    /// denom of coin to mint
    pub mint_denom: Denom,
    /// maximum annual change in inflation rate
    pub inflation_rate_change: Decimal256,
    /// maximum inflation rate
    pub inflation_max: Decimal256,
    /// minimum inflation rate
    pub inflation_min: Decimal256,
    /// goal of percent bonded atoms
    pub goal_bonded: Decimal256,
    /// expected blocks per year
    pub blocks_per_year: u32,
}

impl Default for MintingParams {
    fn default() -> Self {
        Self {
            mint_denom: Denom::from_str(env!("XMOD_STAKING_PARAMS_BOND_DENOM"))
                .expect("default denom for minting is invalid"),
            inflation_rate_change: Default::default(),
            inflation_max: Default::default(),
            inflation_min: Default::default(),
            goal_bonded: Default::default(),
            blocks_per_year: Default::default(),
        }
    }
}

impl ParamsSerialize for MintingParams {
    fn keys() -> HashSet<&'static str> {
        todo!()
    }

    fn to_raw(&self) -> Vec<(&'static str, Vec<u8>)> {
        todo!()
    }
}

impl ParamsDeserialize for MintingParams {
    fn from_raw(_fields: HashMap<&'static str, Vec<u8>>) -> Self {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct MintParamsKeeper<PSK> {
    pub(super) params_subspace_key: PSK,
}

impl<PSK: ParamsSubspaceKey> ParamsKeeper<PSK> for MintParamsKeeper<PSK> {
    type Param = MintingParams;

    fn psk(&self) -> &PSK {
        &self.params_subspace_key
    }

    fn validate(_key: impl AsRef<[u8]>, _value: impl AsRef<[u8]>) -> bool {
        true // TODO
    }
}
