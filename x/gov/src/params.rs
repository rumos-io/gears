use std::{collections::HashMap, str::FromStr, time::Duration};

use gears::{
    application::keepers::params::ParamsKeeper,
    params::{ParamKind, ParamsDeserialize, ParamsSerialize, ParamsSubspaceKey},
    types::{base::coin::Coin, decimal256::Decimal256},
};
use serde::{Deserialize, Serialize};

use crate::errors::{EXISTS, SERDE_JSON_CONVERSION};

const KEY_DEPOSIT_PARAMS: &str = "depositparams";
const KEY_VOTING_PARAMS: &str = "votingparams";
const KEY_TALLY_PARAMS: &str = "tallyparams";

const DEFAULT_PERIOD: Duration = Duration::from_secs(172800); // 2 days

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DepositParams {
    pub min_deposit: Vec<Coin>,       // SendCoins?
    pub max_deposit_period: Duration, // ?
}

impl Default for DepositParams {
    fn default() -> Self {
        Self {
            min_deposit: vec![Coin::from_str("10000000stake").expect("default is valid")],
            max_deposit_period: DEFAULT_PERIOD,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VotingParams {
    pub voting_period: Duration,
}

impl Default for VotingParams {
    fn default() -> Self {
        Self {
            voting_period: DEFAULT_PERIOD,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TallyParams {
    pub quorum: Decimal256,
    pub threshold: Decimal256,
    pub veto_threshold: Decimal256,
}

impl Default for TallyParams {
    fn default() -> Self {
        Self {
            quorum: Decimal256::from_atomics(334_u16, 3).expect("Default should be valid"),
            threshold: Decimal256::from_atomics(5_u8, 1).expect("Default should be valid"),
            veto_threshold: Decimal256::from_atomics(334_u16, 3).expect("Default should be valid"),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GovParams {
    pub deposit: DepositParams,
    pub voting: VotingParams,
    pub tally: TallyParams,
}

impl ParamsSerialize for GovParams {
    fn keys() -> HashMap<&'static str, ParamKind> {
        [
            (KEY_DEPOSIT_PARAMS, ParamKind::Bytes),
            (KEY_VOTING_PARAMS, ParamKind::Bytes),
            (KEY_TALLY_PARAMS, ParamKind::Bytes),
        ]
        .into_iter()
        .collect()
    }

    fn to_raw(&self) -> Vec<(&'static str, Vec<u8>)> {
        let mut map = Vec::new();

        map.push((
            KEY_DEPOSIT_PARAMS,
            serde_json::to_vec(&self.deposit).expect(SERDE_JSON_CONVERSION),
        ));

        map.push((
            KEY_VOTING_PARAMS,
            serde_json::to_vec(&self.voting).expect(SERDE_JSON_CONVERSION),
        ));

        map.push((
            KEY_TALLY_PARAMS,
            serde_json::to_vec(&self.tally).expect(SERDE_JSON_CONVERSION),
        ));

        map
    }
}

impl ParamsDeserialize for GovParams {
    fn from_raw(mut fields: HashMap<&'static str, Vec<u8>>) -> Self {
        Self {
            deposit: serde_json::from_slice(&fields.remove(KEY_DEPOSIT_PARAMS).expect(EXISTS))
                .expect(SERDE_JSON_CONVERSION),
            voting: serde_json::from_slice(&fields.remove(KEY_VOTING_PARAMS).expect(EXISTS))
                .expect(SERDE_JSON_CONVERSION),
            tally: serde_json::from_slice(&fields.remove(KEY_TALLY_PARAMS).expect(EXISTS))
                .expect(SERDE_JSON_CONVERSION),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GovParamsKeeper<PSK: ParamsSubspaceKey> {
    pub params_subspace_key: PSK,
}

impl<PSK: ParamsSubspaceKey> ParamsKeeper<PSK> for GovParamsKeeper<PSK> {
    type Param = GovParams;

    fn psk(&self) -> &PSK {
        &self.params_subspace_key
    }
}
