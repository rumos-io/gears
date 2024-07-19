use std::borrow::Cow;

use gears::{
    ext::PaginationKey,
    tendermint::types::time::Timestamp,
    types::{
        address::{AccAddress, ValAddress},
        decimal256::Decimal256,
        uint::Uint256,
    },
    x::types::delegation::StakingDelegation,
};
use serde::{Deserialize, Serialize};

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
        // TODO: consider to move the DateTime type and work with timestamps into Gears
        // The timestamp is provided by context and conversion won't fail.
        let time = chrono::DateTime::from_timestamp(time.seconds, time.nanos as u32).unwrap();
        let completion_time = chrono::DateTime::from_timestamp(
            self.completion_time.seconds,
            self.completion_time.nanos as u32,
        )
        .unwrap();
        completion_time <= time
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
        // TODO: consider to move the DateTime type and work with timestamps into Gears
        // The timestamp is provided by context and conversion won't fail.
        let time = chrono::DateTime::from_timestamp(time.seconds, time.nanos as u32).unwrap();
        let completion_time = chrono::DateTime::from_timestamp(
            self.completion_time.seconds,
            self.completion_time.nanos as u32,
        )
        .unwrap();
        completion_time <= time
    }
}
