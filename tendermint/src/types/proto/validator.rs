use super::crypto::PublicKey;

#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ValidatorUpdate {
    pub pub_key: Option<PublicKey>,
    pub power: i64,
}

impl From<ValidatorUpdate> for inner::ValidatorUpdate {
    fn from(ValidatorUpdate { pub_key, power }: ValidatorUpdate) -> Self {
        Self {
            pub_key: pub_key.map(Into::into),
            power,
        }
    }
}

impl From<inner::ValidatorUpdate> for ValidatorUpdate {
    fn from(inner::ValidatorUpdate { pub_key, power }: inner::ValidatorUpdate) -> Self {
        Self {
            pub_key: pub_key.map(Into::into),
            power,
        }
    }
}

/// Validator
#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct Validator {
    /// The first 20 bytes of SHA256(public key)
    #[prost(bytes = "bytes", tag = "1")]
    pub address: ::prost::bytes::Bytes,
    /// The voting power
    #[prost(int64, tag = "3")]
    pub power: i64,
}

impl From<Validator> for inner::Validator {
    fn from(Validator { address, power }: Validator) -> Self {
        Self { address, power }
    }
}

impl From<inner::Validator> for Validator {
    fn from(inner::Validator { address, power }: inner::Validator) -> Self {
        Self { address, power }
    }
}

pub(crate) mod inner {
    pub use tendermint_proto::abci::Validator;
    pub use tendermint_proto::abci::ValidatorUpdate;
}
