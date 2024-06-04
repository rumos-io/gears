/// keeper consts
pub(crate) mod keeper {
    pub(crate) const POOL_KEY: [u8; 1] = [0];
    pub(crate) const LAST_TOTAL_POWER_KEY: [u8; 1] = [1];
    pub(crate) const VALIDATORS_KEY: [u8; 1] = [2];
    pub(crate) const LAST_VALIDATOR_POWER_KEY: [u8; 1] = [3];

    pub(crate) const DELEGATIONS_KEY: [u8; 1] = [4];
    pub(crate) const REDELEGATIONS_KEY: [u8; 1] = [5];
    pub(crate) const UNBONDING_DELEGATIONS_KEY: [u8; 1] = [6];

    pub(crate) const VALIDATORS_BY_POWER_INDEX_KEY: [u8; 1] = [7];
    pub(crate) const VALIDATORS_BY_CONS_ADDR_KEY: [u8; 1] = [8];
    pub(crate) const VALIDATORS_QUEUE_KEY: [u8; 1] = [9];
    pub(crate) const UNBONDING_QUEUE_KEY: [u8; 1] = [10];
    pub(crate) const REDELEGATION_QUEUE_KEY: [u8; 1] = [11];
    pub(crate) const REDELEGATION_BY_VAL_DST_INDEX_KEY: [u8; 1] = [12];

    pub(crate) const NOT_BONDED_POOL_NAME: &str = "not_bonded_tokens_pool";
    pub(crate) const BONDED_POOL_NAME: &str = "bonded_tokens_pool";

    pub(crate) const ATTRIBUTE_KEY_AMOUNT: &str = "amount";
    pub(crate) const ATTRIBUTE_KEY_MODULE: &str = "module";
    pub(crate) const ATTRIBUTE_KEY_VALIDATOR: &str = "validator";
    pub(crate) const ATTRIBUTE_KEY_SRC_VALIDATOR: &str = "source_validator";
    pub(crate) const ATTRIBUTE_KEY_DST_VALIDATOR: &str = "destination_validator";
    pub(crate) const ATTRIBUTE_KEY_DELEGATOR: &str = "delegator";
    pub(crate) const ATTRIBUTE_KEY_SENDER: &str = "sender";
    pub(crate) const ATTRIBUTE_KEY_NEW_SHARES: &str = "new_shares";
    pub(crate) const ATTRIBUTE_KEY_COMPLETION_TIME: &str = "completion_time";
    // TODO: check
    pub(crate) const ATTRIBUTE_VALUE_CATEGORY: &str = "staking";

    pub(crate) const EVENT_TYPE_CREATE_VALIDATOR: &str = "create_validator";
    pub(crate) const EVENT_TYPE_COMPLETE_UNBONDING: &str = "complete_unbonding";
    pub(crate) const EVENT_TYPE_COMPLETE_REDELEGATION: &str = "complete_redelegation";
    pub(crate) const EVENT_TYPE_MESSAGE: &str = "message";
    pub(crate) const EVENT_TYPE_DELEGATE: &str = "delegate";
    pub(crate) const EVENT_TYPE_REDELEGATE: &str = "redelegate";
}

pub(crate) mod proto {
    use gears::types::decimal256::Decimal256;

    pub(crate) const MAX_MONIKER_LENGTH: usize = 70;
    pub(crate) const MAX_IDENTITY_LENGTH: usize = 3000;
    pub(crate) const MAX_WEBSITE_LENGTH: usize = 140;
    pub(crate) const MAX_SECURITY_CONTACT_LENGTH: usize = 140;
    pub(crate) const MAX_DETAILS_LENGTH: usize = 280;
    // TODO: check
    pub(crate) const ONE_DEC: Decimal256 = Decimal256::one();
}

pub(crate) mod error {
    pub(crate) const SERDE_ENCODING_DOMAIN_TYPE: &str =
        "serde_json should encode domain type that implements deserialization";
    pub(crate) const TIMESTAMP_NANOS_EXPECT: &str =
        "provided the date is between 1677-09-21T00:12:43.145224192 and 2262-04-11T23:47:16.854775807 the conversion won't fail";
}
