/// key for global distribution state
pub(crate) const FEE_POOL_KEY: [u8; 1] = [0x00];
/// key for the proposer operator address
pub(crate) const PROPOSER_KEY: [u8; 1] = [0x01];
/// key for outstanding rewards
pub(crate) const VALIDATOR_OUTSTANDING_REWARDS_PREFIX: [u8; 1] = [0x02];

/// key for delegator withdraw address
pub(crate) const DELEGATOR_WITHDRAW_ADDR_PREFIX: [u8; 1] = [0x03];
/// key for delegator starting info
pub(crate) const DELEGATOR_STARTING_INFO_PREFIX: [u8; 1] = [0x04];
/// key for historical validators rewards / stake
pub(crate) const VALIDATOR_HISTORICAL_REWARDS_PREFIX: [u8; 1] = [0x05];
/// key for current validator rewards
pub(crate) const VALIDATOR_CURRENT_REWARDS_PREFIX: [u8; 1] = [0x06];
/// key for accumulated validator commission
pub(crate) const VALIDATOR_ACCUMULATED_COMMISSION_PREFIX: [u8; 1] = [0x07];
/// key for validator slash fraction
pub(crate) const VALIDATOR_SLASH_EVENT_PREFIX: [u8; 1] = [0x08];
