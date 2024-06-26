use bytes::Bytes;
use gears::{
    core::errors::CoreError, error::IBC_ENCODE_UNWRAP, tendermint::types::proto::Protobuf,
};
use ibc_proto::google::protobuf::Any;
use prost::Message;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Message)]
pub struct RawTextProposal {
    #[prost(string, tag = "1")]
    pub title: String,
    #[prost(string, tag = "2")]
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextProposal {
    pub title: String,
    pub description: String,
}

impl TextProposal {
    pub const TYPE_URL: &'static str = "/cosmos.params.v1beta1/TextProposal";
}

impl Protobuf<RawTextProposal> for TextProposal {}

impl TryFrom<RawTextProposal> for TextProposal {
    type Error = CoreError;

    fn try_from(
        RawTextProposal { title, description }: RawTextProposal,
    ) -> Result<Self, Self::Error> {
        Ok(Self { title, description })
    }
} 

impl From<TextProposal> for RawTextProposal {
    fn from(TextProposal { title, description }: TextProposal) -> Self {
        Self { title, description }
    }
}

impl TryFrom<Any> for TextProposal {
    type Error = CoreError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        if value.type_url != Self::TYPE_URL {
            Err(CoreError::DecodeGeneral(
                "message type not recognized".into(),
            ))?
        }
        Self::decode::<Bytes>(value.value.into())
            .map_err(|e| CoreError::DecodeProtobuf(e.to_string()))
    }
}

impl From<TextProposal> for Any {
    fn from(msg: TextProposal) -> Self {
        Any {
            type_url: TextProposal::TYPE_URL.to_string(),
            value: msg.encode_vec().expect(IBC_ENCODE_UNWRAP),
        }
    }
}
