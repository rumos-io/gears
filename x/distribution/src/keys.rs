use gears::types::address::{AccAddress, ValAddress};

use crate::{
    DELEGATOR_STARTING_INFO_PREFIX, DELEGATOR_WITHDRAW_ADDR_PREFIX,
    VALIDATOR_ACCUMULATED_COMMISSION_PREFIX, VALIDATOR_CURRENT_REWARDS_PREFIX,
    VALIDATOR_HISTORICAL_REWARDS_PREFIX, VALIDATOR_OUTSTANDING_REWARDS_PREFIX,
    VALIDATOR_SLASH_EVENT_PREFIX,
};

/// delegator_withdraw_addr_key creates the key for a delegator's withdraw addr
pub fn delegator_withdraw_addr_key(addr: AccAddress) -> Vec<u8> {
    [
        DELEGATOR_WITHDRAW_ADDR_PREFIX.to_vec(),
        length_prefixed(addr.len(), addr),
    ]
    .concat()
}

/// delegator_starting_info_key creates the key for a delegator's starting info.
pub fn delegator_starting_info_key(
    validator_address: ValAddress,
    delegator_address: AccAddress,
) -> Vec<u8> {
    [
        DELEGATOR_STARTING_INFO_PREFIX.to_vec(),
        length_prefixed(validator_address.len(), validator_address),
        length_prefixed(delegator_address.len(), delegator_address),
    ]
    .concat()
}

/// validator_outstanding_rewards_key creates the outstanding rewards key for a validator
pub fn validator_outstanding_rewards_key(addr: ValAddress) -> Vec<u8> {
    [
        VALIDATOR_OUTSTANDING_REWARDS_PREFIX.to_vec(),
        length_prefixed(addr.len(), addr),
    ]
    .concat()
}

/// validator_accumulated_commission_key creates the key for a validator's current commission
pub fn validator_accumulated_commission_key(addr: ValAddress) -> Vec<u8> {
    [
        VALIDATOR_ACCUMULATED_COMMISSION_PREFIX.to_vec(),
        length_prefixed(addr.len(), addr),
    ]
    .concat()
}

/// validator_historical_rewards_key creates the key for a validator's historical rewards
pub fn validator_historical_rewards_key(addr: ValAddress, power: u64) -> Vec<u8> {
    [
        VALIDATOR_HISTORICAL_REWARDS_PREFIX.to_vec(),
        length_prefixed(addr.len(), addr),
        power.to_le_bytes().to_vec(),
    ]
    .concat()
}

/// validator_current_rewards_key creates the key for a validator's historical rewards
pub fn validator_current_rewards_key(addr: ValAddress) -> Vec<u8> {
    [
        VALIDATOR_CURRENT_REWARDS_PREFIX.to_vec(),
        length_prefixed(addr.len(), addr),
    ]
    .concat()
}

/// validator_slash_event_key creates the key for a validator's slash fraction
pub fn validator_slash_event_key(addr: ValAddress, height: u64, period: u64) -> Vec<u8> {
    [
        VALIDATOR_SLASH_EVENT_PREFIX.to_vec(),
        length_prefixed(addr.len(), addr),
        height.to_be_bytes().to_vec(),
        period.to_be_bytes().to_vec(),
    ]
    .concat()
}

// private function for addresses
fn length_prefixed(len: u8, addr: impl AsRef<[u8]>) -> Vec<u8> {
    [vec![len], addr.as_ref().to_vec()].concat()
}
