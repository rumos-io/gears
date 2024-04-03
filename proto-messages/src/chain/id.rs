use std::{fmt::Display, str::FromStr};

use ibc::core::host::types::{error::IdentifierError, identifiers::ChainId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Id {
    inner_id: ChainId,
}

impl Id {
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> Result<Self, ChainIdParseError> {
        Ok(Self {
            inner_id: ChainId::from_str(s.as_ref())?,
        })
    }

    /// Extract the revision number from the chain identifier
    pub fn revision_number(&self) -> u64 {
        self.inner_id.revision_number()
    }
}

impl FromStr for Id {
    type Err = ChainIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl From<ChainId> for Id {
    fn from(inner_id: ChainId) -> Self {
        Self { inner_id }
    }
}

impl From<Id> for ChainId {
    fn from(value: Id) -> Self {
        value.inner_id
    }
}

impl From<tendermint::informal::chain::Id> for Id {
    fn from(value: tendermint::informal::chain::Id) -> Self {
        Self {
            inner_id: ChainId::from_str(value.as_str()).expect("Two chain id should be valid"),
        }
    }
}

impl From<Id> for tendermint::informal::chain::Id {
    fn from(value: Id) -> Self {
        tendermint::informal::chain::Id::from_str(value.as_ref())
            .expect("Two chain id should be valid")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
#[error("Failed to parse id: {0}")]
pub struct ChainIdParseError(pub String);

impl From<IdentifierError> for ChainIdParseError {
    fn from(value: IdentifierError) -> Self {
        Self(value.to_string())
    }
}

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        self.inner_id.as_str()
    }
}

impl TryFrom<String> for Id {
    type Error = ChainIdParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self {
            inner_id: ChainId::from_str(&value)?,
        })
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner_id)
    }
}
