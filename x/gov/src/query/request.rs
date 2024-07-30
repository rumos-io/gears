use anyhow::anyhow;
use gears::{
    core::errors::CoreError,
    derive::{Protobuf, Query},
    error::ProtobufError,
    tendermint::types::proto::Protobuf,
    types::{address::AccAddress, pagination::request::PaginationRequest},
};

use crate::types::proposal::ProposalStatus;

pub mod inner {
    pub use ibc_proto::cosmos::gov::v1beta1::QueryDepositRequest;
    pub use ibc_proto::cosmos::gov::v1beta1::QueryDepositsRequest;
    pub use ibc_proto::cosmos::gov::v1beta1::QueryParamsRequest;
    pub use ibc_proto::cosmos::gov::v1beta1::QueryProposalRequest;
    pub use ibc_proto::cosmos::gov::v1beta1::QueryProposalsRequest;
    pub use ibc_proto::cosmos::gov::v1beta1::QueryTallyResultRequest;
    pub use ibc_proto::cosmos::gov::v1beta1::QueryVoteRequest;
    pub use ibc_proto::cosmos::gov::v1beta1::QueryVotesRequest;

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct QueryAllParamsRequest {}

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct QueryProposerRequest {
        #[prost(uint64, tag = "1")]
        pub proposal_id: u64,
    }
}

#[derive(Clone, PartialEq, Debug, Query, Protobuf)]
#[query(url = "/cosmos.gov.v1beta1.Query/Proposal")]
#[proto(raw = "inner::QueryProposalRequest")]
pub struct QueryProposalRequest {
    pub proposal_id: u64,
}

#[derive(Clone, PartialEq, Debug, Query)]
#[query(url = "/cosmos.gov.v1beta1.Query/Proposals")]
// #[proto(raw = "inner::QueryProposalsRequest")]
pub struct QueryProposalsRequest {
    pub voter: Option<AccAddress>,
    pub depositor: Option<AccAddress>,
    pub proposal_status: Option<ProposalStatus>,
    pub pagination: Option<PaginationRequest>,
}

impl TryFrom<inner::QueryProposalsRequest> for QueryProposalsRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryProposalsRequest {
            proposal_status,
            voter,
            depositor,
            pagination,
        }: inner::QueryProposalsRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            voter: match voter.is_empty() {
                true => None,
                false => Some(
                    AccAddress::from_bech32(&voter)
                        .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
                ),
            },
            depositor: match depositor.is_empty() {
                true => None,
                false => Some(
                    AccAddress::from_bech32(&depositor)
                        .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
                ),
            },
            proposal_status: match proposal_status <= -1 {
                true => None,
                false => Some(proposal_status.try_into()?),
            },
            pagination: pagination.map(|var| var.into()),
        })
    }
}

impl From<QueryProposalsRequest> for inner::QueryProposalsRequest {
    fn from(
        QueryProposalsRequest {
            voter,
            depositor,
            proposal_status,
            pagination: _,
        }: QueryProposalsRequest,
    ) -> Self {
        Self {
            proposal_status: proposal_status.map(|this| this as i32).unwrap_or(-1),
            voter: voter.map(|this| this.to_string()).unwrap_or_default(),
            depositor: depositor.map(|this| this.to_string()).unwrap_or_default(),
            pagination: None,
        }
    }
}

impl Protobuf<inner::QueryProposalsRequest> for QueryProposalsRequest {}

#[derive(Clone, PartialEq, Debug, Query, Protobuf)]
#[query(url = "/cosmos.gov.v1beta1.Query/Vote")]
#[proto(raw = "inner::QueryVoteRequest")]
pub struct QueryVoteRequest {
    pub proposal_id: u64,
    pub voter: AccAddress,
}

#[derive(Clone, PartialEq, Debug, Query, Protobuf)]
#[query(url = "/cosmos.gov.v1beta1.Query/Votes")]
#[proto(raw = "inner::QueryVotesRequest")]
pub struct QueryVotesRequest {
    pub proposal_id: u64,
    #[proto(optional)]
    pub pagination: Option<PaginationRequest>,
}

#[derive(Clone, PartialEq, Debug, Query, Protobuf)]
#[query(url = "/cosmos.gov.v1beta1.Query/Param")]
#[proto(raw = "inner::QueryParamsRequest")]
pub struct QueryParamsRequest {
    #[proto(name = "params_type")]
    pub kind: ParamsQuery,
}

#[derive(Clone, PartialEq, Debug, strum::EnumString, strum::Display)]
pub enum ParamsQuery {
    #[strum(serialize = "tallying")]
    Tally,
    #[strum(serialize = "voting")]
    Voting,
    #[strum(serialize = "deposit")]
    Deposit,
}

impl TryFrom<String> for ParamsQuery {
    type Error = ProtobufError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self::try_from(value.as_str()).map_err(|e| anyhow!("{e}"))?)
    }
}

impl From<ParamsQuery> for String {
    fn from(value: ParamsQuery) -> Self {
        value.to_string()
    }
}

#[derive(Clone, PartialEq, Debug, Query, Protobuf)]
#[query(url = "/cosmos.gov.v1beta1.Query/Deposit")]
#[proto(raw = "inner::QueryDepositRequest")]
pub struct QueryDepositRequest {
    pub proposal_id: u64,
    pub depositor: AccAddress,
}

#[derive(Clone, PartialEq, Debug, Query, Protobuf)]
#[query(url = "/cosmos.gov.v1beta1.Query/Deposits")]
#[proto(raw = "inner::QueryDepositsRequest")]
pub struct QueryDepositsRequest {
    pub proposal_id: u64,
    #[proto(optional)]
    pub pagination: Option<PaginationRequest>,
}

#[derive(Clone, PartialEq, Debug, Query, Protobuf)]
#[query(url = "/cosmos.gov.v1beta1.Query/Tally")]
#[proto(raw = "inner::QueryTallyResultRequest")]
pub struct QueryTallyResultRequest {
    pub proposal_id: u64,
}

#[derive(Clone, PartialEq, Debug, Query, Protobuf)]
#[query(url = "/cosmos.gov.v1beta1.Query/Params")]
#[proto(raw = "inner::QueryAllParamsRequest")]
pub struct QueryAllParamsRequest;

#[derive(Clone, PartialEq, Debug, Query, Protobuf)]
#[query(url = "/cosmos.gov.v1beta1.Query/Proposer")]
#[proto(raw = "inner::QueryProposerRequest")]
pub struct QueryProposerRequest {
    pub proposal_id: u64,
}
