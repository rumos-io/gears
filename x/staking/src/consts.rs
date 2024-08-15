/// keeper consts
pub(crate) mod keeper {
    pub(crate) const LAST_VALIDATOR_POWER_KEY: [u8; 1] = [17];
    pub(crate) const LAST_TOTAL_POWER_KEY: [u8; 1] = [18];

    pub(crate) const VALIDATORS_KEY: [u8; 1] = [33];
    pub(crate) const VALIDATORS_BY_CONS_ADDR_KEY: [u8; 1] = [34];
    pub(crate) const VALIDATORS_BY_POWER_INDEX_KEY: [u8; 1] = [35];

    pub(crate) const DELEGATION_KEY: [u8; 1] = [49];
    pub(crate) const UNBONDING_DELEGATION_KEY: [u8; 1] = [50];
    pub(crate) const REDELEGATION_KEY: [u8; 1] = [52];
    pub(crate) const REDELEGATION_BY_VAL_SRC_INDEX_KEY: [u8; 1] = [0x35];
    pub(crate) const REDELEGATION_BY_VAL_DST_INDEX_KEY: [u8; 1] = [0x36];

    pub(crate) const UNBONDING_QUEUE_KEY: [u8; 1] = [65];
    pub(crate) const REDELEGATION_QUEUE_KEY: [u8; 1] = [66];
    pub(crate) const VALIDATOR_QUEUE_KEY: [u8; 1] = [67];

    pub(crate) const HISTORICAL_INFO_KEY: [u8; 1] = [80];

    /// Constants to refer name in module declaration
    pub const NOT_BONDED_POOL_NAME: &str = "not_bonded_tokens_pool";
    pub const BONDED_POOL_NAME: &str = "bonded_tokens_pool";

    pub(crate) const ATTRIBUTE_KEY_MODULE: &str = "module";
    pub(crate) const ATTRIBUTE_KEY_SENDER: &str = "sender";
    pub(crate) const ATTRIBUTE_KEY_AMOUNT: &str = "amount";
    pub(crate) const ATTRIBUTE_KEY_COMMISSION_RATE: &str = "commission_rate";
    pub(crate) const ATTRIBUTE_KEY_MIN_SELF_DELEGATION: &str = "min_self_delegation";

    pub(crate) const ATTRIBUTE_KEY_VALIDATOR: &str = "validator";
    pub(crate) const ATTRIBUTE_KEY_SRC_VALIDATOR: &str = "source_validator";
    pub(crate) const ATTRIBUTE_KEY_DST_VALIDATOR: &str = "destination_validator";
    pub(crate) const ATTRIBUTE_KEY_DELEGATOR: &str = "delegator";
    pub(crate) const ATTRIBUTE_KEY_NEW_SHARES: &str = "new_shares";
    pub(crate) const ATTRIBUTE_KEY_COMPLETION_TIME: &str = "completion_time";
    // TODO: check
    pub(crate) const ATTRIBUTE_VALUE_CATEGORY: &str = "staking";

    pub(crate) const EVENT_TYPE_CREATE_VALIDATOR: &str = "create_validator";
    pub(crate) const EVENT_TYPE_EDIT_VALIDATOR: &str = "edit_validator";
    pub(crate) const EVENT_TYPE_COMPLETE_UNBONDING: &str = "complete_unbonding";
    pub(crate) const EVENT_TYPE_COMPLETE_REDELEGATION: &str = "complete_redelegation";
    pub(crate) const EVENT_TYPE_MESSAGE: &str = "message";
    pub(crate) const EVENT_TYPE_DELEGATE: &str = "delegate";
    pub(crate) const EVENT_TYPE_REDELEGATE: &str = "redelegate";
    pub(crate) const EVENT_TYPE_UNBOND: &str = "unbond";
}

pub(crate) mod proto {
    pub(crate) const MAX_MONIKER_LENGTH: usize = 70;
    pub(crate) const MAX_IDENTITY_LENGTH: usize = 3000;
    pub(crate) const MAX_WEBSITE_LENGTH: usize = 140;
    pub(crate) const MAX_SECURITY_CONTACT_LENGTH: usize = 140;
    pub(crate) const MAX_DETAILS_LENGTH: usize = 280;
}

pub(crate) mod error {
    pub(crate) const SERDE_ENCODING_DOMAIN_TYPE: &str =
        "serde_json should encode domain type that implements deserialization";
}
