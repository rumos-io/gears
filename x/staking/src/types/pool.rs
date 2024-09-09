use gears::{
    core::Protobuf,
    types::{errors::StdError, uint::Uint256},
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

mod inner {
    pub use ibc_proto::cosmos::staking::v1beta1::Pool;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pool {
    pub bonded_tokens: Uint256,
    pub not_bonded_tokens: Uint256,
}

impl TryFrom<inner::Pool> for Pool {
    type Error = StdError;

    fn try_from(
        inner::Pool {
            not_bonded_tokens,
            bonded_tokens,
        }: inner::Pool,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            bonded_tokens: Uint256::from_str(&bonded_tokens)?,
            not_bonded_tokens: Uint256::from_str(&not_bonded_tokens)?,
        })
    }
}

impl From<Pool> for inner::Pool {
    fn from(
        Pool {
            bonded_tokens,
            not_bonded_tokens,
        }: Pool,
    ) -> Self {
        Self {
            not_bonded_tokens: not_bonded_tokens.to_string(),
            bonded_tokens: bonded_tokens.to_string(),
        }
    }
}

impl Protobuf<inner::Pool> for Pool {}
