use crate::Validator;
use gears::{
    core::{errors::CoreError, Protobuf},
    tendermint::{self, types::proto::header::Header},
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// HistoricalInfo contains header and validator information for a given block.
/// It is stored as part of staking module's state, which persists the `n` most
/// recent HistoricalInfo
/// (`n` is set by the staking module's `historical_entries` parameter).
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct HistoricalInfo {
    pub header: Header,
    pub validators: Vec<Validator>,
}

impl HistoricalInfo {
    /// Method will create a historical information struct from header and valset
    /// it will first sort valset before inclusion into historical info
    pub fn new(
        header: Header,
        mut validators: Vec<Validator>,
        power_reduction: u64,
    ) -> HistoricalInfo {
        fn less(v1: &Validator, v2: &Validator, power_reduction: u64) -> Ordering {
            let cons_power1 = v1.consensus_power(power_reduction);
            let cons_power2 = v2.consensus_power(power_reduction);
            if cons_power1 == cons_power2 {
                let addr1 = Vec::from(v1.cons_addr());
                let addr2 = Vec::from(v2.cons_addr());
                addr1.cmp(&addr2)
            } else {
                cons_power1.cmp(&cons_power2)
            }
        }
        validators.sort_by(|v1, v2| less(v1, v2, power_reduction));
        HistoricalInfo { header, validators }
    }
}

mod inner {
    pub use gears::tendermint::types::proto::header::HistoricalInfo;
}

impl From<HistoricalInfo> for inner::HistoricalInfo {
    fn from(historical_info: HistoricalInfo) -> Self {
        inner::HistoricalInfo {
            header: Some(historical_info.header.into()),
            validators: historical_info
                .validators
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl TryFrom<inner::HistoricalInfo> for HistoricalInfo {
    type Error = CoreError;

    fn try_from(value: inner::HistoricalInfo) -> Result<Self, Self::Error> {
        let header = value
            .header
            .ok_or(CoreError::MissingField("Missing field 'header'.".into()))?
            .try_into()
            .map_err(|e: tendermint::error::Error| CoreError::DecodeGeneral(e.to_string()))?;
        Ok(HistoricalInfo {
            header,
            validators: value
                .validators
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<Validator>, _>>()?,
        })
    }
}

impl Protobuf<inner::HistoricalInfo> for HistoricalInfo {}
