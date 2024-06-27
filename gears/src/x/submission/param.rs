use crate::{
    core::errors::CoreError, error::IBC_ENCODE_UNWRAP, params::ParamsSubspaceKey,
    tendermint::types::proto::Protobuf,
};
use bytes::Bytes;
use ibc_proto::google::protobuf::Any;
use prost::Message;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Message)]
pub struct RawParamChange {
    #[prost(string, tag = "1")]
    pub subspace: String,
    #[prost(bytes, tag = "2")]
    pub key: Vec<u8>,
    #[prost(bytes, tag = "3")]
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParamChange<PSK> {
    pub subspace: PSK,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

impl<PSK> ParamChange<PSK> {
    pub const TYPE_URL: &'static str = "/cosmos.params.v1beta1/ParamChange";
}

impl<PSK: ParamsSubspaceKey> Protobuf<RawParamChange> for ParamChange<PSK> {}

impl<PSK: ParamsSubspaceKey> TryFrom<RawParamChange> for ParamChange<PSK> {
    type Error = CoreError;

    fn try_from(
        RawParamChange {
            subspace,
            key,
            value,
        }: RawParamChange,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            subspace: PSK::from_str(&subspace)
                .map_err(|e| CoreError::DecodeGeneral(e.to_string()))?,
            key,
            value,
        })
    }
}

impl<PSK: ParamsSubspaceKey> From<ParamChange<PSK>> for RawParamChange {
    fn from(
        ParamChange {
            subspace,
            key,
            value,
        }: ParamChange<PSK>,
    ) -> Self {
        Self {
            subspace: subspace.name().to_owned(),
            key,
            value,
        }
    }
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
            value: msg.encode_vec().expect(IBC_ENCODE_UNWRAP),
        }
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct RawParameterChangeProposal {
    #[prost(string, tag = "1")]
    pub title: String,
    #[prost(string, tag = "2")]
    pub description: String,
    #[prost(repeated, message, tag = "3")]
    pub changes: Vec<RawParamChange>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterChangeProposal<PSK> {
    pub title: String,
    pub description: String,
    pub changes: Vec<ParamChange<PSK>>,
}

impl<PSK> ParameterChangeProposal<PSK> {
    pub const TYPE_URL: &'static str = "/cosmos.params.v1beta1/ParameterChangeProposal";
}

impl<PSK: ParamsSubspaceKey> Protobuf<RawParameterChangeProposal> for ParameterChangeProposal<PSK> {}

impl<PSK: ParamsSubspaceKey> TryFrom<RawParameterChangeProposal> for ParameterChangeProposal<PSK> {
    type Error = CoreError;

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
            value: msg.encode_vec().expect(IBC_ENCODE_UNWRAP),
        }
    }
}
