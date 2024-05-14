use crate::error::Error;

use super::crypto::PublicKey;

#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ValidatorUpdate {
    pub pub_key: PublicKey,
    pub power: i64,
}

impl From<ValidatorUpdate> for inner::ValidatorUpdate {
    fn from(ValidatorUpdate { pub_key, power }: ValidatorUpdate) -> Self {
        Self {
            pub_key: Some(pub_key.into()),
            power,
        }
    }
}

impl TryFrom<inner::ValidatorUpdate> for ValidatorUpdate {
    type Error = Error;

    fn try_from(
        inner::ValidatorUpdate { pub_key, power }: inner::ValidatorUpdate,
    ) -> Result<Self, Self::Error> {
        let pub_key = pub_key.ok_or(Error::InvalidData("public key is empty".to_string()))?;
        Ok(Self {
            pub_key: pub_key.try_into()?,
            power,
        })
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
