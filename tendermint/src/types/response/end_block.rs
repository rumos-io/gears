use crate::types::proto::{consensus::ConsensusParams, event::Event, validator::ValidatorUpdate};

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct ResponseEndBlock {
    #[prost(message, repeated, tag = "1")]
    pub validator_updates: Vec<ValidatorUpdate>,
    #[prost(message, optional, tag = "2")]
    pub consensus_param_updates: Option<ConsensusParams>,
    #[prost(message, repeated, tag = "3")]
    pub events: Vec<Event>,
}

impl From<super::inner::ResponseEndBlock> for ResponseEndBlock {
    fn from(
        super::inner::ResponseEndBlock {
            validator_updates,
            consensus_param_updates,
            events,
        }: super::inner::ResponseEndBlock,
    ) -> Self {
        Self {
            validator_updates: validator_updates.into_iter().map(Into::into).collect(),
            consensus_param_updates: consensus_param_updates.map(Into::into),
            events: events.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<ResponseEndBlock> for super::inner::ResponseEndBlock {
    fn from(
        ResponseEndBlock {
            validator_updates,
            consensus_param_updates,
            events,
        }: ResponseEndBlock,
    ) -> Self {
        Self {
            validator_updates: validator_updates.into_iter().map(Into::into).collect(),
            consensus_param_updates: consensus_param_updates.map(Into::into),
            events: events.into_iter().map(Into::into).collect(),
        }
    }
}
