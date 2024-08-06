use gears::core::any::google::Any;
use serde::{Deserialize, Serialize};

use crate::Evidence;

/// GenesisState defines the evidence module's genesis state.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct GenesisState {
    /// evidence defines all the evidence at genesis.
    pub evidence: Vec<Any>,
}

impl GenesisState {
    // TODO: compatibility with sdk. Maybe we may omit it
    pub fn validate<T: Evidence + Default>(&self) -> anyhow::Result<()> {
        for e in &self.evidence {
            let evidence = T::decode(e.value.as_slice())?;
            evidence.validate_basic()?;
        }
        Ok(())
    }
}
