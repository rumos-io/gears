/// keeper consts
pub(crate) mod keeper {
    pub(crate) const POOL_KEY: [u8; 1] = [0];
    pub(crate) const LAST_TOTAL_POWER_KEY: [u8; 1] = [1];
    pub(crate) const VALIDATORS_KEY: [u8; 1] = [2];
    pub(crate) const LAST_VALIDATOR_POWER_KEY: [u8; 1] = [3];
    pub(crate) const DELEGATIONS_KEY: [u8; 1] = [4];
    pub(crate) const REDELEGATIONS_KEY: [u8; 1] = [5];
    pub(crate) const VALIDATORS_BY_POWER_INDEX_KEY: [u8; 1] = [6];
    pub(crate) const VALIDATORS_BY_CONS_ADDR_KEY: [u8; 1] = [7];
    pub(crate) const VALIDATORS_QUEUE_KEY: [u8; 1] = [8];
    pub(crate) const UBD_QUEUE_KEY: [u8; 1] = [9];
    pub(crate) const UNBONDING_QUEUE_KEY: [u8; 1] = [10];
    pub(crate) const REDELEGATION_QUEUE_KEY: [u8; 1] = [11];

    pub(crate) const NOT_BONDED_POOL_NAME: &str = "not_bonded_tokens_pool";
    pub(crate) const BONDED_POOL_NAME: &str = "bonded_tokens_pool";
    pub(crate) const EVENT_TYPE_COMPLETE_UNBONDING: &str = "complete_unbonding";
    pub(crate) const EVENT_TYPE_COMPLETE_REDELEGATION: &str = "complete_redelegation";
    pub(crate) const ATTRIBUTE_KEY_AMOUNT: &str = "amount";
    pub(crate) const ATTRIBUTE_KEY_VALIDATOR: &str = "validator";
    pub(crate) const ATTRIBUTE_KEY_DELEGATOR: &str = "delegator";
}

pub(crate) mod proto {
    pub(crate) const MAX_MONIKER_LENGTH: usize = 70;
    pub(crate) const MAX_IDENTITY_LENGTH: usize = 3000;
    pub(crate) const MAX_WEBSITE_LENGTH: usize = 140;
    pub(crate) const MAX_SECURITY_CONTACT_LENGTH: usize = 140;
    pub(crate) const MAX_DETAILS_LENGTH: usize = 280;
}

pub(crate) mod expect {
    pub(crate) const SERDE_DECODING_DOMAIN_TYPE: &str =
        "serde_json should decode domain type that implements deserialization";
    pub(crate) const SERDE_ENCODING_DOMAIN_TYPE: &str =
        "serde_json should encode domain type that implements deserialization";
}
