use crate::types::proto::event::Event;

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct ResponseBeginBlock {
    #[prost(message, repeated, tag = "1")]
    pub events: Vec<Event>,
}

impl From<ResponseBeginBlock> for super::inner::ResponseBeginBlock {
    fn from(ResponseBeginBlock { events }: ResponseBeginBlock) -> Self {
        Self {
            events: events.into_iter().map(Into::into).collect(),
        }
    }
}
