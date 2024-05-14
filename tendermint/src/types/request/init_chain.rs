use crate::types::{
    proto::{consensus::ConsensusParams, validator::ValidatorUpdate},
    time::Timestamp,
};

#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RequestInitChain {
    pub time: Option<Timestamp>,
    pub chain_id: String,
    pub consensus_params: Option<ConsensusParams>,
    pub validators: ::prost::alloc::vec::Vec<ValidatorUpdate>,
    pub app_state_bytes: ::prost::bytes::Bytes,
    pub initial_height: i64,
}

impl From<RequestInitChain> for super::inner::RequestInitChain {
    fn from(
        RequestInitChain {
            time,
            chain_id,
            consensus_params,
            validators,
            app_state_bytes,
            initial_height,
        }: RequestInitChain,
    ) -> Self {
        Self {
            time: time.map(Into::into),
            chain_id,
            consensus_params: consensus_params.map(Into::into),
            validators: validators.into_iter().map(Into::into).collect(),
            app_state_bytes,
            initial_height,
        }
    }
}

impl From<super::inner::RequestInitChain> for RequestInitChain {
    fn from(
        super::inner::RequestInitChain {
            time,
            chain_id,
            consensus_params,
            validators,
            app_state_bytes,
            initial_height,
        }: super::inner::RequestInitChain,
    ) -> Self {
        Self {
            time: time.map(Into::into),
            chain_id,
            consensus_params: consensus_params.map(Into::into),
            validators: validators.into_iter().map(Into::into).collect(),
            app_state_bytes,
            initial_height,
        }
    }
}
