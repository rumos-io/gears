use crate::types::{
    proto::{consensus::ConsensusParams, validator::ValidatorUpdate},
    time::Timestamp,
};

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct RequestInitChain {
    #[prost(message, optional, tag = "1")]
    pub time: Option<Timestamp>,
    #[prost(string, tag = "2")]
    pub chain_id: String,
    #[prost(message, optional, tag = "3")]
    pub consensus_params: Option<ConsensusParams>,
    #[prost(message, repeated, tag = "4")]
    pub validators: ::prost::alloc::vec::Vec<ValidatorUpdate>,
    #[prost(bytes = "bytes", tag = "5")]
    pub app_state_bytes: ::prost::bytes::Bytes,
    #[prost(int64, tag = "6")]
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
