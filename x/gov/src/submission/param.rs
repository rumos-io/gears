use bytes::Bytes;
use gears::{
    core::{errors::CoreError, Protobuf},
    derive::{Protobuf, Raw},
    error::ProtobufError,
    params::ParamsSubspaceKey,
};
use ibc_proto::google::protobuf::Any;
use prost::Message;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Raw, Protobuf)]
#[raw(derive(Serialize, Deserialize, Clone, PartialEq))]
pub struct ParamChange<PSK: ParamsSubspaceKey> {
    #[raw(kind(string), raw = String)]
    #[proto(
        from = "PSK::from_subspace_str",
        from_ref,
        into = "PSK::name",
        into_ref
    )]
    pub subspace: PSK,
    #[raw(kind(bytes))]
    #[proto(repeated)]
    pub key: Vec<u8>,
    #[raw(kind(bytes))]
    #[proto(repeated)]
    pub value: Vec<u8>,
}

impl<PSK: ParamsSubspaceKey> ParamChange<PSK> {
    pub const TYPE_URL: &'static str = "/cosmos.params.v1beta1/ParamChange";
}

impl<PSK: ParamsSubspaceKey> TryFrom<Any> for ParamChange<PSK> {
    type Error = CoreError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        if value.type_url != Self::TYPE_URL {
            Err(CoreError::DecodeGeneral(
                "message type not recognized".into(),
            ))?
        }
        ParamChange::decode::<Bytes>(value.value.into())
            .map_err(|e| CoreError::DecodeProtobuf(e.to_string()))
    }
}

impl<PSK: ParamsSubspaceKey> From<ParamChange<PSK>> for Any {
    fn from(msg: ParamChange<PSK>) -> Self {
        Any {
            type_url: ParamChange::<PSK>::TYPE_URL.to_string(),
            value: msg.encode_vec(),
        }
    }
}

#[derive(Clone, PartialEq, Message, Serialize, Deserialize)]
pub struct RawParameterChangeProposal {
    #[prost(string, tag = "1")]
    pub title: String,
    #[prost(string, tag = "2")]
    pub description: String,
    #[prost(repeated, message, tag = "3")]
    pub changes: Vec<RawParamChange>,
}

impl From<RawParameterChangeProposal> for Any {
    fn from(msg: RawParameterChangeProposal) -> Self {
        Any {
            type_url: "/cosmos.params.v1beta1/ParameterChangeProposal".to_owned(),
            value: msg.encode_to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterChangeProposal<PSK: ParamsSubspaceKey> {
    pub title: String,
    pub description: String,
    pub changes: Vec<ParamChange<PSK>>,
}

impl<PSK: ParamsSubspaceKey> ParameterChangeProposal<PSK> {
    pub const TYPE_URL: &'static str = "/cosmos.params.v1beta1/ParameterChangeProposal";
}

impl<PSK: ParamsSubspaceKey> Protobuf<RawParameterChangeProposal> for ParameterChangeProposal<PSK> {}

impl<PSK: ParamsSubspaceKey> TryFrom<RawParameterChangeProposal> for ParameterChangeProposal<PSK> {
    type Error = ProtobufError;

    fn try_from(
        RawParameterChangeProposal {
            title,
            description,
            changes,
        }: RawParameterChangeProposal,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            title,
            description,
            changes: {
                let mut result = Vec::with_capacity(changes.len());
                for change in changes {
                    result.push(change.try_into()?)
                }
                result
            },
        })
    }
}

impl<PSK: ParamsSubspaceKey> From<ParameterChangeProposal<PSK>> for RawParameterChangeProposal {
    fn from(
        ParameterChangeProposal {
            title,
            description,
            changes,
        }: ParameterChangeProposal<PSK>,
    ) -> Self {
        Self {
            title,
            description,
            changes: changes.into_iter().map(|e| e.into()).collect(),
        }
    }
}

impl<PSK: ParamsSubspaceKey> TryFrom<Any> for ParameterChangeProposal<PSK> {
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

impl<PSK: ParamsSubspaceKey> From<ParameterChangeProposal<PSK>> for Any {
    fn from(msg: ParameterChangeProposal<PSK>) -> Self {
        Any {
            type_url: ParameterChangeProposal::<PSK>::TYPE_URL.to_string(),
            value: msg.encode_vec(),
        }
    }
}
