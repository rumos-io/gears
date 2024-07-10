use address::ValAddress;
use serde::{Deserialize, Serialize};

use crate::error::Error;

use super::crypto::PublicKey;

const MAX_VALIDATOR_POWER: u64 = 8198552921648689607;
/// VotingPower holds validator power and guarantees that its value is less than
/// 8198552921648689607.
// TODO: can be a Copy type
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct VotingPower {
    // look at https://github.com/tendermint/tendermint/issues/2985
    // https://github.com/tendermint/tendermint/issues/7913
    power: u64,
}

impl VotingPower {
    pub fn new(power: u64) -> Result<VotingPower, Error> {
        if power > MAX_VALIDATOR_POWER {
            return Err(Error::InvalidData(format!(
                "validator power is greater than max validator power {MAX_VALIDATOR_POWER}"
            )));
        }

        Ok(VotingPower { power })
    }

    pub fn power(&self) -> u64 {
        self.power
    }
}

impl TryFrom<u64> for VotingPower {
    type Error = Error;

    fn try_from(power: u64) -> Result<Self, Self::Error> {
        Self::new(power)
    }
}

impl From<VotingPower> for u64 {
    fn from(power: VotingPower) -> Self {
        power.power()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct ValidatorUpdate {
    pub pub_key: PublicKey,
    pub power: VotingPower,
}

impl From<ValidatorUpdate> for inner::ValidatorUpdate {
    fn from(ValidatorUpdate { pub_key, power }: ValidatorUpdate) -> Self {
        Self {
            pub_key: Some(pub_key.into()),
            power: power.power() as i64,
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
            power: VotingPower::new(power as u64)?,
        })
    }
}

/// Validator
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validator {
    /// The first 20 bytes of SHA256(public key)
    pub address: ValAddress,
    /// The voting power
    pub power: VotingPower,
}

impl From<Validator> for inner::Validator {
    fn from(Validator { address, power }: Validator) -> Self {
        Self {
            address: address.as_ref().to_vec().into(),
            power: power.power() as i64,
        }
    }
}

impl TryFrom<inner::Validator> for Validator {
    type Error = Error;
    fn try_from(
        inner::Validator { address, power }: inner::Validator,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: ValAddress::try_from(address.to_vec())
                .map_err(|e| Error::InvalidData(e.to_string()))?,
            // SAFETY:
            // https://github.com/tendermint/tendermint/blob/9c236ffd6c56add84f3c17930ae75c26c68d61ec/types/validator_set.go#L15-L22
            power: VotingPower::new(power as u64)?,
        })
    }
}

pub(crate) mod inner {
    pub use tendermint_proto::abci::Validator;
    pub use tendermint_proto::abci::ValidatorUpdate;
}
