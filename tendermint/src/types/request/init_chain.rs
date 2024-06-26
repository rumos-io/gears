use serde::de::DeserializeOwned;

use crate::{
    error::Error,
    types::{
        chain_id::ChainId,
        proto::{consensus::ConsensusParams, validator::ValidatorUpdate},
        time::Timestamp,
    },
};

#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RequestInitChain<G> {
    pub time: Timestamp,
    pub chain_id: ChainId,
    pub consensus_params: ConsensusParams,
    pub validators: Vec<ValidatorUpdate>,
    pub app_genesis: G,
    pub initial_height: i64, //TODO: use u64?
}

impl<G: DeserializeOwned> TryFrom<super::inner::RequestInitChain> for RequestInitChain<G> {
    type Error = Error;

    fn try_from(
        super::inner::RequestInitChain {
            time,
            chain_id,
            consensus_params,
            validators,
            app_state_bytes,
            initial_height,
        }: super::inner::RequestInitChain,
    ) -> Result<Self, Self::Error> {
        let app_genesis: G = serde_json::from_slice(&app_state_bytes)
            .map_err(|e| Error::InvalidData(format!("invalid app_state_bytes: {e}")))?;

        Ok(Self {
            time: time
                .ok_or(Error::InvalidData("time is empty".to_string()))?
                .into(),
            chain_id: chain_id
                .parse()
                .map_err(|e| Self::Error::InvalidData(format!("invalid chain_id: {e}")))?,
            consensus_params: consensus_params
                .ok_or(Error::InvalidData("consensus params is empty".to_string()))?
                .try_into()?,
            validators: validators
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<ValidatorUpdate>, Error>>()?,
            app_genesis,
            initial_height,
        })
    }
}
