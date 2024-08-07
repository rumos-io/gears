use std::borrow::Cow;

use gears::{
    core::{errors::CoreError, Protobuf},
    ext::PaginationKey,
    tendermint::types::time::timestamp::Timestamp,
    types::{
        address::{AccAddress, ValAddress},
        decimal256::{CosmosDecimalProtoString, Decimal256},
        uint::Uint256,
    },
    x::types::delegation::StakingDelegation,
};
use serde::{Deserialize, Serialize};

use crate::consts::error::SERDE_ENCODING_DOMAIN_TYPE;

/// Delegation represents the bond with tokens held by an account. It is
/// owned by one delegator, and is associated with the voting power of one
/// validator.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Delegation {
    pub delegator_address: AccAddress,
    pub validator_address: ValAddress,
    pub shares: Decimal256,
}

impl StakingDelegation for Delegation {
    fn delegator(&self) -> &AccAddress {
        &self.delegator_address
    }

    fn validator(&self) -> &ValAddress {
        &self.validator_address
    }

    fn shares(&self) -> &Decimal256 {
        &self.shares
    }
}

impl TryFrom<Vec<u8>> for Delegation {
    type Error = CoreError;

    fn try_from(raw: Vec<u8>) -> Result<Self, Self::Error> {
        serde_json::from_slice(&raw).map_err(|e| CoreError::DecodeGeneral(e.to_string()))
    }
}

impl From<Delegation> for Vec<u8> {
    fn from(value: Delegation) -> Self {
        serde_json::to_vec(&value).expect(SERDE_ENCODING_DOMAIN_TYPE)
    }
}

impl From<Delegation> for inner::Delegation {
    fn from(value: Delegation) -> Self {
        inner::Delegation {
            delegator_address: value.delegator_address.to_string(),
            validator_address: value.validator_address.to_string(),
            shares: value.shares.to_cosmos_proto_string(),
        }
    }
}

impl TryFrom<inner::Delegation> for Delegation {
    type Error = CoreError;

    fn try_from(proto: inner::Delegation) -> Result<Self, Self::Error> {
        Ok(Delegation {
            delegator_address: AccAddress::from_bech32(&proto.delegator_address).map_err(|e| {
                CoreError::DecodeGeneral(format!("delegator_address: {}", e.to_string()))
            })?,
            validator_address: ValAddress::from_bech32(&proto.validator_address).map_err(|e| {
                CoreError::DecodeGeneral(format!("validator_address: {}", e.to_string()))
            })?,
            shares: Decimal256::from_cosmos_proto_string(&proto.shares)
                .map_err(|e| CoreError::DecodeGeneral(format!("shares: {}", e.to_string())))?,
        })
    }
}

impl Protobuf<inner::Delegation> for Delegation {}

mod inner {
    pub use ibc_proto::cosmos::staking::v1beta1::Delegation;
}

/// Delegation represents the bond with tokens held by an account. It is
/// owned by one delegator, and is associated with the voting power of one
/// validator.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct UnbondingDelegation {
    pub delegator_address: AccAddress,
    pub validator_address: ValAddress,
    pub entries: Vec<UnbondingDelegationEntry>,
}

/// UnbondingDelegationEntry - entry to an UnbondingDelegation
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct UnbondingDelegationEntry {
    pub creation_height: u32,
    pub completion_time: Timestamp,
    pub initial_balance: Uint256,
    pub balance: Uint256,
}

impl UnbondingDelegationEntry {
    pub fn is_mature(&self, time: &Timestamp) -> bool {
        self.completion_time <= *time
    }
}

/// Redelegation contains the list of a particular delegator's
/// redelegating bonds from a particular source validator to a
/// particular destination validator
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Redelegation {
    pub delegator_address: AccAddress,
    pub validator_src_address: ValAddress,
    pub validator_dst_address: ValAddress,
    pub entries: Vec<RedelegationEntry>,
}

impl Redelegation {
    pub fn add_entry(&mut self, redelegation_entry: RedelegationEntry) {
        self.entries.push(redelegation_entry);
    }
}

impl PaginationKey for Redelegation {
    fn iterator_key(&self) -> Cow<'_, [u8]> {
        Cow::Owned(
            [
                self.delegator_address.to_string().as_bytes(),
                self.validator_src_address.to_string().as_bytes(),
                self.validator_dst_address.to_string().as_bytes(),
            ]
            .concat(),
        )
    }
}

/// RedelegationEntry - entry to a Redelegation
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RedelegationEntry {
    pub creation_height: u32,
    pub completion_time: Timestamp,
    pub initial_balance: Uint256,
    pub share_dst: Decimal256,
}

impl RedelegationEntry {
    pub fn is_mature(&self, time: &Timestamp) -> bool {
        self.completion_time <= *time
    }
}
