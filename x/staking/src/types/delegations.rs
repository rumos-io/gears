use std::borrow::Cow;

use gears::{
    core::{errors::CoreError, Protobuf},
    extensions::pagination::PaginationKey,
    tendermint::types::time::timestamp::Timestamp,
    types::{
        address::{AccAddress, ValAddress},
        decimal256::{CosmosDecimalProtoString, Decimal256},
        uint::Uint256,
    },
    x::types::delegation::StakingDelegation,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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
            delegator_address: AccAddress::from_bech32(&proto.delegator_address)
                .map_err(|e| CoreError::DecodeGeneral(format!("delegator_address: {}", e)))?,
            validator_address: ValAddress::from_bech32(&proto.validator_address)
                .map_err(|e| CoreError::DecodeGeneral(format!("validator_address: {}", e)))?,
            shares: Decimal256::from_cosmos_proto_string(&proto.shares)
                .map_err(|e| CoreError::DecodeGeneral(format!("shares: {}", e)))?,
        })
    }
}

impl Protobuf<inner::Delegation> for Delegation {}

mod inner {
    pub use ibc_proto::cosmos::staking::v1beta1::Delegation;
    pub use ibc_proto::cosmos::staking::v1beta1::Redelegation;
    pub use ibc_proto::cosmos::staking::v1beta1::RedelegationEntry;
    pub use ibc_proto::cosmos::staking::v1beta1::UnbondingDelegation;
    pub use ibc_proto::cosmos::staking::v1beta1::UnbondingDelegationEntry;
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

impl From<UnbondingDelegation> for inner::UnbondingDelegation {
    fn from(value: UnbondingDelegation) -> Self {
        inner::UnbondingDelegation {
            delegator_address: value.delegator_address.to_string(),
            validator_address: value.validator_address.to_string(),
            entries: value.entries.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<inner::UnbondingDelegation> for UnbondingDelegation {
    type Error = CoreError;

    fn try_from(proto: inner::UnbondingDelegation) -> Result<Self, Self::Error> {
        Ok(UnbondingDelegation {
            delegator_address: AccAddress::from_bech32(&proto.delegator_address)
                .map_err(|e| CoreError::DecodeGeneral(format!("delegator_address: {}", e)))?,
            validator_address: ValAddress::from_bech32(&proto.validator_address)
                .map_err(|e| CoreError::DecodeGeneral(format!("validator_address: {}", e)))?,
            entries: proto
                .entries
                .into_iter()
                .map(TryFrom::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl Protobuf<inner::UnbondingDelegation> for UnbondingDelegation {}

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

impl From<UnbondingDelegationEntry> for inner::UnbondingDelegationEntry {
    fn from(value: UnbondingDelegationEntry) -> Self {
        inner::UnbondingDelegationEntry {
            creation_height: value.creation_height.into(),
            completion_time: Some(value.completion_time.into()),
            initial_balance: value.initial_balance.into(),
            balance: value.balance.into(),
        }
    }
}

impl TryFrom<inner::UnbondingDelegationEntry> for UnbondingDelegationEntry {
    type Error = CoreError;

    fn try_from(proto: inner::UnbondingDelegationEntry) -> Result<Self, Self::Error> {
        Ok(UnbondingDelegationEntry {
            creation_height: proto
                .creation_height
                .try_into()
                .map_err(|e| CoreError::DecodeGeneral(format!("creation_height: {}", e)))?,
            completion_time: proto
                .completion_time
                .ok_or(CoreError::DecodeGeneral("completion_time".to_string()))?
                .try_into()
                .map_err(|e| CoreError::DecodeGeneral(format!("completion_time: {}", e)))?,
            initial_balance: Uint256::from_str(&proto.initial_balance)
                .map_err(|e| CoreError::DecodeGeneral(e.to_string()))?,
            balance: Uint256::from_str(&proto.balance)
                .map_err(|e| CoreError::DecodeGeneral(e.to_string()))?,
        })
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

impl From<Redelegation> for inner::Redelegation {
    fn from(value: Redelegation) -> Self {
        inner::Redelegation {
            delegator_address: value.delegator_address.to_string(),
            validator_src_address: value.validator_src_address.to_string(),
            validator_dst_address: value.validator_dst_address.to_string(),
            entries: value.entries.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<inner::Redelegation> for Redelegation {
    type Error = CoreError;

    fn try_from(proto: inner::Redelegation) -> Result<Self, Self::Error> {
        Ok(Redelegation {
            delegator_address: AccAddress::from_bech32(&proto.delegator_address)
                .map_err(|e| CoreError::DecodeGeneral(format!("delegator_address: {}", e)))?,
            validator_src_address: ValAddress::from_bech32(&proto.validator_src_address)
                .map_err(|e| CoreError::DecodeGeneral(format!("validator_src_address: {}", e)))?,
            validator_dst_address: ValAddress::from_bech32(&proto.validator_dst_address)
                .map_err(|e| CoreError::DecodeGeneral(format!("validator_dst_address: {}", e)))?,
            entries: proto
                .entries
                .into_iter()
                .map(TryFrom::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl Protobuf<inner::Redelegation> for Redelegation {}

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

impl From<RedelegationEntry> for inner::RedelegationEntry {
    fn from(value: RedelegationEntry) -> Self {
        inner::RedelegationEntry {
            creation_height: value.creation_height.into(),
            completion_time: Some(value.completion_time.into()),
            initial_balance: value.initial_balance.into(),
            shares_dst: value.share_dst.to_cosmos_proto_string(),
        }
    }
}

impl TryFrom<inner::RedelegationEntry> for RedelegationEntry {
    type Error = CoreError;

    fn try_from(proto: inner::RedelegationEntry) -> Result<Self, Self::Error> {
        Ok(RedelegationEntry {
            creation_height: proto
                .creation_height
                .try_into()
                .map_err(|e| CoreError::DecodeGeneral(format!("creation_height: {}", e)))?,
            completion_time: proto
                .completion_time
                .ok_or(CoreError::DecodeGeneral("completion_time".to_string()))?
                .try_into()
                .map_err(|e| CoreError::DecodeGeneral(format!("completion_time: {}", e)))?,
            initial_balance: Uint256::from_str(&proto.initial_balance)
                .map_err(|e| CoreError::DecodeGeneral(e.to_string()))?,
            share_dst: Decimal256::from_cosmos_proto_string(&proto.shares_dst)
                .map_err(|e| CoreError::DecodeGeneral(format!("shares_dst: {}", e)))?,
        })
    }
}
