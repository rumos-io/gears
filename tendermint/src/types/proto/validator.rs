use address::{AddressError, ValAddress};

use crate::error::Error;

use super::crypto::PublicKey;

#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Debug)]
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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Validator {
    /// The first 20 bytes of SHA256(public key)
    pub address: ValAddress,
    /// The voting power
    pub power: i64,
}

impl From<Validator> for inner::Validator {
    fn from(Validator { address, power }: Validator) -> Self {
        Self {
            address: address.as_ref().to_vec().into(),
            power,
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
            power,
        })
    }
}

pub(crate) mod inner {
    pub use tendermint_proto::abci::Validator;
    pub use tendermint_proto::abci::ValidatorUpdate;
}
