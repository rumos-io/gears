use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use gears::{
    application::keepers::params::ParamsKeeper,
    derive::{Protobuf, Raw},
    extensions::corruption::UnwrapCorrupt,
    params::{ParamsDeserialize, ParamsSerialize, ParamsSubspaceKey},
    types::{
        decimal256::{CosmosDecimalProtoString, Decimal256},
        denom::Denom,
    },
};
use serde::{Deserialize, Serialize};

const MINT_DENOM_KEY: &str = "MintDenom";
const INFLATION_RATE_CHANGE_KEY: &str = "InflationRateChange";
const INFLATION_MAX_KEY: &str = "InflationMax";
const INFLATION_MIN_KEY: &str = "InflationMin";
const GOAL_BONDED_KEY: &str = "GoalBonded";
const BLOCKS_PER_YEAR_KEY: &str = "BlocksPerYear";

#[derive(Debug, Clone, Serialize, Deserialize, Raw, Protobuf)]
pub struct MintParams {
    /// denom of coin to mint
    #[raw(kind(string), raw = String)]
    pub mint_denom: Denom,
    /// maximum annual change in inflation rate
    #[raw(kind(string), raw = String)]
    #[proto(
        from = "CosmosDecimalProtoString::from_cosmos_proto_string",
        from_ref,
        into = "CosmosDecimalProtoString::to_cosmos_proto_string",
        into_ref
    )]
    pub inflation_rate_change: Decimal256,
    /// maximum inflation rate
    #[raw(kind(string), raw = String)]
    #[proto(
        from = "CosmosDecimalProtoString::from_cosmos_proto_string",
        from_ref,
        into = "CosmosDecimalProtoString::to_cosmos_proto_string",
        into_ref
    )]
    pub inflation_max: Decimal256,
    /// minimum inflation rate
    #[raw(kind(string), raw = String)]
    #[proto(
        from = "CosmosDecimalProtoString::from_cosmos_proto_string",
        from_ref,
        into = "CosmosDecimalProtoString::to_cosmos_proto_string",
        into_ref
    )]
    pub inflation_min: Decimal256,
    /// goal of percent bonded atoms
    #[raw(kind(string), raw = String)]
    #[proto(
        from = "CosmosDecimalProtoString::from_cosmos_proto_string",
        from_ref,
        into = "CosmosDecimalProtoString::to_cosmos_proto_string",
        into_ref
    )]
    pub goal_bonded: Decimal256,
    /// expected blocks per year
    #[raw(kind(uint32))]
    pub blocks_per_year: u32,
}

impl Default for MintParams {
    fn default() -> Self {
        Self {
            mint_denom: Denom::from_str(env!("XMOD_STAKING_PARAMS_BOND_DENOM"))
                .expect("default denom for minting is invalid"),
            inflation_rate_change: Decimal256::from_atomics(13_u8, 2).expect("default is valid"),
            inflation_max: Decimal256::from_atomics(20_u8, 2).expect("default is valid"),
            inflation_min: Decimal256::from_atomics(7_u8, 2).expect("default is valid"),
            goal_bonded: Decimal256::from_atomics(67_u8, 2).expect("default is valid"),
            blocks_per_year: 60 * 60 * 8766 / 5, // assuming 5 second block times
        }
    }
}

impl ParamsSerialize for MintParams {
    fn keys() -> HashSet<&'static str> {
        HashSet::from_iter([
            MINT_DENOM_KEY,
            INFLATION_RATE_CHANGE_KEY,
            INFLATION_MAX_KEY,
            INFLATION_MIN_KEY,
            GOAL_BONDED_KEY,
            BLOCKS_PER_YEAR_KEY,
        ])
    }

    fn to_raw(&self) -> Vec<(&'static str, Vec<u8>)> {
        vec![
            (MINT_DENOM_KEY, self.mint_denom.to_string().into_bytes()),
            (
                INFLATION_RATE_CHANGE_KEY,
                self.inflation_rate_change
                    .to_cosmos_proto_string()
                    .into_bytes(),
            ),
            (
                INFLATION_MAX_KEY,
                self.inflation_max.to_cosmos_proto_string().into_bytes(),
            ),
            (
                INFLATION_MIN_KEY,
                self.inflation_min.to_cosmos_proto_string().into_bytes(),
            ),
            (
                GOAL_BONDED_KEY,
                self.goal_bonded.to_cosmos_proto_string().into_bytes(),
            ),
            (
                BLOCKS_PER_YEAR_KEY,
                self.blocks_per_year.to_string().into_bytes(),
            ),
        ]
    }
}

impl ParamsDeserialize for MintParams {
    fn from_raw(fields: HashMap<&'static str, Vec<u8>>) -> Self {
        Self {
            mint_denom: Denom::from_str(&String::from_utf8_lossy(
                fields.get(MINT_DENOM_KEY).unwrap_or_corrupt(),
            ))
            .unwrap_or_corrupt(),
            inflation_rate_change: Decimal256::from_cosmos_proto_string(&String::from_utf8_lossy(
                fields.get(INFLATION_RATE_CHANGE_KEY).unwrap_or_corrupt(),
            ))
            .unwrap_or_corrupt(),
            inflation_max: Decimal256::from_cosmos_proto_string(&String::from_utf8_lossy(
                fields.get(INFLATION_MAX_KEY).unwrap_or_corrupt(),
            ))
            .unwrap_or_corrupt(),
            inflation_min: Decimal256::from_cosmos_proto_string(&String::from_utf8_lossy(
                fields.get(INFLATION_MIN_KEY).unwrap_or_corrupt(),
            ))
            .unwrap_or_corrupt(),
            goal_bonded: Decimal256::from_cosmos_proto_string(&String::from_utf8_lossy(
                fields.get(GOAL_BONDED_KEY).unwrap_or_corrupt(),
            ))
            .unwrap_or_corrupt(),
            blocks_per_year: u32::from_str(&String::from_utf8_lossy(
                fields.get(BLOCKS_PER_YEAR_KEY).unwrap_or_corrupt(),
            ))
            .unwrap_or_corrupt(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MintParamsKeeper<PSK> {
    pub(super) params_subspace_key: PSK,
}

impl<PSK: ParamsSubspaceKey> ParamsKeeper<PSK> for MintParamsKeeper<PSK> {
    type Param = MintParams;

    fn psk(&self) -> &PSK {
        &self.params_subspace_key
    }

    fn validate(key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> bool {
        match String::from_utf8_lossy(key.as_ref()).as_ref() {
            MINT_DENOM_KEY => Denom::from_str(&String::from_utf8_lossy(value.as_ref())).is_ok(),
            INFLATION_RATE_CHANGE_KEY => {
                Decimal256::from_cosmos_proto_string(&String::from_utf8_lossy(value.as_ref()))
                    .is_ok()
            }
            INFLATION_MAX_KEY => {
                Decimal256::from_cosmos_proto_string(&String::from_utf8_lossy(value.as_ref()))
                    .is_ok()
            }
            INFLATION_MIN_KEY => {
                Decimal256::from_cosmos_proto_string(&String::from_utf8_lossy(value.as_ref()))
                    .is_ok()
            }
            GOAL_BONDED_KEY => {
                Decimal256::from_cosmos_proto_string(&String::from_utf8_lossy(value.as_ref()))
                    .is_ok()
            }
            BLOCKS_PER_YEAR_KEY => u32::from_str(&String::from_utf8_lossy(value.as_ref())).is_ok(),
            _ => false,
        }
    }
}
