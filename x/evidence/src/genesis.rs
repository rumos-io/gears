use crate::{errors::GenesisStateError, Evidence};
use gears::core::any::google::Any;
use serde::{Deserialize, Serialize};

/// GenesisState defines the evidence module's genesis state.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct GenesisState<E: Evidence>
where
    <E as std::convert::TryFrom<Any>>::Error: std::fmt::Debug,
{
    /// evidence defines all the evidence at genesis.
    pub evidence: Evidences<E>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(try_from = "Vec<Any>")]
pub struct Evidences<E: Evidence>(Vec<E>)
where
    <E as std::convert::TryFrom<Any>>::Error: std::fmt::Debug;

impl<E: Evidence> IntoIterator for Evidences<E>
where
    <E as std::convert::TryFrom<Any>>::Error: std::fmt::Debug,
{
    type Item = E;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<E: Evidence> TryFrom<Vec<Any>> for Evidences<E>
where
    <E as std::convert::TryFrom<Any>>::Error: std::fmt::Debug,
{
    type Error = GenesisStateError;

    fn try_from(values: Vec<Any>) -> Result<Self, Self::Error> {
        let mut evidences = vec![];
        for v in values {
            let evidence: E = v.try_into().map_err(|_| GenesisStateError::Decode)?;
            evidences.push(evidence);
        }

        Ok(Self(evidences))
    }
}
