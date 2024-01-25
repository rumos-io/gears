use ibc_proto::cosmos::tx::v1beta1::{
    mode_info::{Single, Sum},
    ModeInfo as RawModeInfo,
};
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModeInfo {
    pub sum: Sum,
}

impl Default for ModeInfo {
    fn default() -> Self {
        ModeInfo {
            sum: Sum::Single(Single { mode: 1 }),
        }
    }
}

impl TryFrom<RawModeInfo> for ModeInfo {
    type Error = Error;

    fn try_from(raw: RawModeInfo) -> Result<Self, Self::Error> {
        Ok(ModeInfo {
            sum: raw.sum.ok_or(Error::MissingField(String::from("sum")))?,
        })
    }
}

impl From<ModeInfo> for RawModeInfo {
    fn from(mode_info: ModeInfo) -> RawModeInfo {
        RawModeInfo {
            sum: Some(mode_info.sum),
        }
    }
}

// impl Protobuf<RawModeInfo> for ModeInfo {}
