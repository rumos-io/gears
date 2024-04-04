use crate::types::proto::{
    header::Header,
    info::{Evidence, LastCommitInfo},
};

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct RequestBeginBlock {
    #[prost(bytes = "bytes", tag = "1")]
    pub hash: ::prost::bytes::Bytes,
    #[prost(message, optional, tag = "2")]
    pub header: Option<Header>,
    #[prost(message, optional, tag = "3")]
    pub last_commit_info: Option<LastCommitInfo>,
    #[prost(message, repeated, tag = "4")]
    pub byzantine_validators: Vec<Evidence>,
}

impl From<RequestBeginBlock> for super::inner::RequestBeginBlock {
    fn from(
        RequestBeginBlock {
            hash,
            header,
            last_commit_info,
            byzantine_validators,
        }: RequestBeginBlock,
    ) -> Self {
        Self {
            hash,
            header: header.map(Into::into),
            last_commit_info: last_commit_info.map(Into::into),
            byzantine_validators: byzantine_validators.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<super::inner::RequestBeginBlock> for RequestBeginBlock {
    fn from(
        super::inner::RequestBeginBlock {
            hash,
            header,
            last_commit_info,
            byzantine_validators,
        }: super::inner::RequestBeginBlock,
    ) -> Self {
        Self {
            hash,
            header: header.map(Into::into),
            last_commit_info: last_commit_info.map(Into::into),
            byzantine_validators: byzantine_validators.into_iter().map(Into::into).collect(),
        }
    }
}
