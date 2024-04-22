use crate::types::proto::{consensus::ConsensusParams, validator::ValidatorUpdate};

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct ResponseInitChain {
    #[prost(message, optional, tag = "1")]
    pub consensus_params: Option<ConsensusParams>,
    #[prost(message, repeated, tag = "2")]
    pub validators: Vec<ValidatorUpdate>,
    #[prost(bytes = "bytes", tag = "3")]
    pub app_hash: ::prost::bytes::Bytes,
}

impl From<ResponseInitChain> for super::inner::ResponseInitChain {
    fn from(
        ResponseInitChain {
            consensus_params,
            validators,
            app_hash,
        }: ResponseInitChain,
    ) -> Self {
        Self {
            consensus_params: consensus_params.map(Into::into),
            validators: validators.into_iter().map(Into::into).collect(),
            app_hash,
        }
    }
}
