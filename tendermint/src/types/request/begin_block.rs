use crate::types::proto::{
    header::Header,
    info::{Evidence, LastCommitInfo},
};

#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RequestBeginBlock {
    pub hash: ::prost::bytes::Bytes,
    pub header: Header,
    pub last_commit_info: Option<LastCommitInfo>,
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
            header: Some(header.into()),
            last_commit_info: last_commit_info.map(Into::into),
            byzantine_validators: byzantine_validators.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<super::inner::RequestBeginBlock> for RequestBeginBlock {
    type Error = crate::error::Error;

    fn try_from(
        super::inner::RequestBeginBlock {
            hash,
            header,
            last_commit_info,
            byzantine_validators,
        }: super::inner::RequestBeginBlock,
    ) -> Result<Self, Self::Error> {
        let last_commit_info = if let Some(info) = last_commit_info {
            Some(info.try_into()?)
        } else {
            None
        };
        Ok(Self {
            hash,
            header: header
                .ok_or_else(|| crate::error::Error::InvalidData("header is missing".into()))?
                .try_into()?,
            last_commit_info,
            byzantine_validators: byzantine_validators.into_iter().map(Into::into).collect(),
        })
    }
}
