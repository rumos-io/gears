use address::{AddressError, ValAddress};
use serde::{Deserialize, Serialize};
use ux::u63;

use crate::error::Error;

use super::crypto::PublicKey;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
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

// TODO: copy from the gears
fn serialize_number_to_string<T, S>(x: &T, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: std::string::ToString,
{
    s.serialize_str(&x.to_string())
}

fn deserialize_u63_from_string<'de, D>(deserializer: D) -> Result<u63, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Number(u64),
    }

    match StringOrInt::deserialize(deserializer)? {
        StringOrInt::String(s) => Ok(u63::new(
            s.parse::<u64>().map_err(serde::de::Error::custom)?,
        )),
        StringOrInt::Number(i) => Ok(u63::new(i)),
    }
}

/// Validator
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validator {
    /// The first 20 bytes of SHA256(public key)
    pub address: ValAddress,
    /// The voting power
    // look at https://github.com/tendermint/tendermint/issues/2985
    // https://github.com/tendermint/tendermint/issues/7913
    #[serde(serialize_with = "serialize_number_to_string")]
    #[serde(deserialize_with = "deserialize_u63_from_string")]
    pub power: u63,
}

impl From<Validator> for inner::Validator {
    fn from(Validator { address, power }: Validator) -> Self {
        Self {
            address: address.as_ref().to_vec().into(),
            power: u64::from(power) as i64,
        }
    }
}

impl TryFrom<inner::Validator> for Validator {
    type Error = AddressError;
    fn try_from(
        inner::Validator { address, power }: inner::Validator,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: ValAddress::try_from(address.to_vec())?,
            // SAFETY:
            // https://github.com/tendermint/tendermint/blob/9c236ffd6c56add84f3c17930ae75c26c68d61ec/types/validator_set.go#L15-L22
            power: u63::new(power as u64),
        })
    }
}

pub(crate) mod inner {
    pub use tendermint_proto::abci::Validator;
    pub use tendermint_proto::abci::ValidatorUpdate;
}
