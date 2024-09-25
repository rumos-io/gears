use anyhow::anyhow;
use gears::{
    application::keepers::params::ParamsKeeper,
    extensions::corruption::UnwrapCorrupt,
    params::{ParamKind, ParamsDeserialize, ParamsSerialize, ParamsSubspaceKey},
    tendermint::types::time::duration::Duration,
    types::denom::Denom,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

mod environment;

mod inner {
    pub use ibc_proto::cosmos::staking::v1beta1::Params;
}

const KEY_UNBONDING_TIME: &str = "UnbondingTime";
const KEY_MAX_VALIDATORS: &str = "MaxValidators";
const KEY_MAX_ENTRIES: &str = "MaxEntries";
const KEY_HISTORICAL_ENTRIES: &str = "HistoricalEntries";
const KEY_BOND_DENOM: &str = "BondDenom";

/// ['Params'] defines the parameters for the staking module. The params are guaranteed to be valid:
/// - unbonding_time is non negative
/// - max_validators is positive
/// - max_entries is positive
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct StakingParams {
    pub unbonding_time: Duration,
    pub max_validators: u32,
    pub max_entries: u32,
    pub historical_entries: u32,
    pub bond_denom: Denom,
}

impl TryFrom<inner::Params> for StakingParams {
    type Error = anyhow::Error;

    fn try_from(
        inner::Params {
            unbonding_time,
            max_validators,
            max_entries,
            historical_entries,
            bond_denom,
            min_commission_rate: _,
        }: inner::Params,
    ) -> Result<Self, Self::Error> {
        StakingParams::new(
            i128::from(
                Duration::try_from(
                    unbonding_time.ok_or(anyhow!("missing field 'unbonding_time'"))?,
                )
                .map_err(|_| anyhow!("cannot conver google duration"))?
                .duration_nanoseconds(),
            )
            .try_into()
            .map_err(|_| anyhow!("cannot conver google duration"))?,
            max_validators,
            max_entries,
            historical_entries,
            bond_denom.try_into()?,
        )
    }
}

impl From<StakingParams> for inner::Params {
    fn from(
        StakingParams {
            unbonding_time,
            max_validators,
            max_entries,
            historical_entries,
            bond_denom,
        }: StakingParams,
    ) -> Self {
        inner::Params {
            unbonding_time: Some(unbonding_time.into()),
            max_validators,
            max_entries,
            historical_entries,
            bond_denom: bond_denom.to_string(),
            min_commission_rate: "0.0".to_string(),
        }
    }
}

impl Default for StakingParams {
    fn default() -> Self {
        let bond_denom =
            Denom::try_from(environment::DEFAULT_DENOM).expect("default denom should be valid");
        StakingParams {
            // 3 weeks
            unbonding_time: Duration::new_from_nanos(60_000_000_000 * 60 * 24 * 7 * 3),
            max_validators: 100,
            max_entries: 7,
            bond_denom,
            historical_entries: 10_000,
        }
    }
}

impl ParamsSerialize for StakingParams {
    fn keys() -> HashSet<&'static str> {
        [
            KEY_UNBONDING_TIME,
            KEY_MAX_VALIDATORS,
            KEY_MAX_ENTRIES,
            KEY_HISTORICAL_ENTRIES,
            KEY_BOND_DENOM,
        ]
        .into_iter()
        .collect()
    }

    fn to_raw(&self) -> Vec<(&'static str, Vec<u8>)> {
        vec![
            (
                KEY_UNBONDING_TIME,
                format!(
                    "\"{}\"",
                    i128::from(self.unbonding_time.duration_nanoseconds())
                )
                .into_bytes(),
            ),
            (
                KEY_MAX_VALIDATORS,
                self.max_validators.to_string().into_bytes(),
            ),
            (KEY_MAX_ENTRIES, self.max_entries.to_string().into_bytes()),
            (
                KEY_HISTORICAL_ENTRIES,
                self.historical_entries.to_string().into_bytes(),
            ),
            (
                KEY_BOND_DENOM,
                format!("\"{}\"", self.bond_denom).into_bytes(),
            ),
        ]
    }
}

impl ParamsDeserialize for StakingParams {
    fn from_raw(mut fields: HashMap<&'static str, Vec<u8>>) -> Self {
        let unbonding_time = ParamKind::I64
            .parse_param(fields.remove(KEY_UNBONDING_TIME).unwrap_or_corrupt())
            .signed_64()
            .expect("param serialized as i64 should be deserialized without errors");
        let max_validators =
            String::from_utf8(fields.remove(KEY_MAX_VALIDATORS).unwrap_or_corrupt())
                .expect("should be valid utf-8")
                .parse::<u32>()
                .expect("should be valid u32");
        let max_entries = String::from_utf8(fields.remove(KEY_MAX_ENTRIES).unwrap_or_corrupt())
            .expect("should be valid utf-8")
            .parse::<u32>()
            .expect("should be valid u32");
        let historical_entries =
            String::from_utf8(fields.remove(KEY_HISTORICAL_ENTRIES).unwrap_or_corrupt())
                .expect("should be valid utf-8")
                .parse::<u32>()
                .expect("should be valid u32");
        let bond_denom = ParamKind::String
            .parse_param(fields.remove(KEY_BOND_DENOM).unwrap_or_corrupt())
            .string()
            .expect("param serialized as string should be deserialized without errors")
            .strip_prefix('\"')
            .unwrap_or_corrupt()
            .strip_suffix('\"')
            .unwrap_or_corrupt()
            .try_into()
            .unwrap_or_corrupt();

        // TODO: should we validate the params here?

        StakingParams {
            unbonding_time: Duration::new_from_nanos(unbonding_time),
            max_validators,
            max_entries,
            bond_denom,
            historical_entries,
        }
    }
}

impl StakingParams {
    pub fn new(
        unbonding_time: i64,
        max_validators: u32,
        max_entries: u32,
        historical_entries: u32,
        bond_denom: Denom,
    ) -> Result<Self, anyhow::Error> {
        if unbonding_time < 0 {
            return Err(anyhow::anyhow!(format!(
                "unbonding time must be non negative: {}",
                unbonding_time
            )));
        }

        if max_validators == 0 {
            return Err(anyhow::anyhow!(format!(
                "max validators must be positive: {}",
                max_validators
            )));
        }

        if max_entries == 0 {
            return Err(anyhow::anyhow!(format!(
                "max entries must be positive: {}",
                max_entries
            )));
        }

        Ok(StakingParams {
            unbonding_time: Duration::new_from_nanos(unbonding_time),
            max_validators,
            max_entries,
            bond_denom,
            historical_entries,
        })
    }

    pub fn unbonding_time(&self) -> Duration {
        self.unbonding_time
    }

    pub fn max_validators(&self) -> u32 {
        self.max_validators
    }

    pub fn max_entries(&self) -> u32 {
        self.max_entries
    }

    pub fn historical_entries(&self) -> u32 {
        self.historical_entries
    }

    pub fn bond_denom(&self) -> &Denom {
        &self.bond_denom
    }
}

#[derive(Debug, Clone)]
pub struct StakingParamsKeeper<PSK: ParamsSubspaceKey> {
    pub params_subspace_key: PSK,
}
impl<PSK: ParamsSubspaceKey> ParamsKeeper<PSK> for StakingParamsKeeper<PSK> {
    type Param = StakingParams;

    fn psk(&self) -> &PSK {
        &self.params_subspace_key
    }

    fn validate(key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> bool {
        match String::from_utf8_lossy(key.as_ref()).as_ref() {
            KEY_UNBONDING_TIME => ParamKind::I64
                .parse_param(value.as_ref().to_vec())
                .signed_64()
                .is_some(),
            KEY_MAX_VALIDATORS => ParamKind::U32
                .parse_param(value.as_ref().to_vec())
                .signed_64()
                .is_some(),
            KEY_MAX_ENTRIES => ParamKind::U32
                .parse_param(value.as_ref().to_vec())
                .signed_64()
                .is_some(),
            KEY_HISTORICAL_ENTRIES => ParamKind::U32
                .parse_param(value.as_ref().to_vec())
                .signed_64()
                .is_some(),
            KEY_BOND_DENOM => ParamKind::String
                .parse_param(value.as_ref().to_vec())
                .string()
                .is_some(),

            _ => false,
        }
    }
}
