use gears::{
    core::errors::CoreError,
    derive::Query,
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

#[derive(Clone, PartialEq, Debug, Query)]
#[query(
    url = "/cosmos.gov.v1beta1.Query/Proposal",
    raw = "inner::QueryProposalRequest"
)]
pub struct QueryProposalRequest {
    pub proposal_id: u64,
}

impl TryFrom<inner::QueryProposalRequest> for QueryProposalRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryProposalRequest { proposal_id }: inner::QueryProposalRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self { proposal_id })
    }
}

impl From<QueryProposalRequest> for inner::QueryProposalRequest {
    fn from(QueryProposalRequest { proposal_id }: QueryProposalRequest) -> Self {
        Self { proposal_id }
    }
}

#[derive(Clone, PartialEq, Debug, Query)]
#[query(
    url = "/cosmos.gov.v1beta1.Query/Proposals",
    raw = "inner::QueryProposalsRequest"
)]
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

#[derive(Clone, PartialEq, Debug, Query)]
#[query(
    url = "/cosmos.gov.v1beta1.Query/Vote",
    raw = "inner::QueryVoteRequest"
)]
pub struct QueryVoteRequest {
    pub proposal_id: u64,
    pub voter: AccAddress,
}

impl TryFrom<inner::QueryVoteRequest> for QueryVoteRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryVoteRequest { proposal_id, voter }: inner::QueryVoteRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proposal_id,
            voter: AccAddress::from_bech32(&voter)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
        })
    }
}

impl From<QueryVoteRequest> for inner::QueryVoteRequest {
    fn from(QueryVoteRequest { proposal_id, voter }: QueryVoteRequest) -> Self {
        Self {
            proposal_id,
            voter: voter.to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Query)]
#[query(
    url = "/cosmos.gov.v1beta1.Query/Votes",
    raw = "inner::QueryVotesRequest"
)]
pub struct QueryVotesRequest {
    pub proposal_id: u64,
    pub pagination: Option<PaginationRequest>,
}

impl TryFrom<inner::QueryVotesRequest> for QueryVotesRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryVotesRequest {
            proposal_id,
            pagination,
        }: inner::QueryVotesRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proposal_id,
            pagination: pagination.map(|e| e.into()),
        })
    }
}

impl From<QueryVotesRequest> for inner::QueryVotesRequest {
    fn from(
        QueryVotesRequest {
            proposal_id,
            pagination: _,
        }: QueryVotesRequest,
    ) -> Self {
        Self {
            proposal_id,
            pagination: None,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Query)]
#[query(
    url = "/cosmos.gov.v1beta1.Query/Param",
    raw = "inner::QueryParamsRequest"
)]
pub struct QueryParamsRequest {
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

impl TryFrom<inner::QueryParamsRequest> for QueryParamsRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryParamsRequest { params_type }: inner::QueryParamsRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: params_type.parse().map_err(|_| {
                CoreError::DecodeGeneral("failed to parse `params_type`".to_owned())
            })?,
        })
    }
}

impl From<QueryParamsRequest> for inner::QueryParamsRequest {
    fn from(value: QueryParamsRequest) -> Self {
        Self {
            params_type: value.kind.to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Query)]
#[query(
    url = "/cosmos.gov.v1beta1.Query/Deposit",
    raw = "inner::QueryDepositRequest"
)]
pub struct QueryDepositRequest {
    pub proposal_id: u64,
    pub depositor: AccAddress,
}

impl TryFrom<inner::QueryDepositRequest> for QueryDepositRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryDepositRequest {
            proposal_id,
            depositor,
        }: inner::QueryDepositRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proposal_id,
            depositor: AccAddress::from_bech32(&depositor)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
        })
    }
}

impl From<QueryDepositRequest> for inner::QueryDepositRequest {
    fn from(
        QueryDepositRequest {
            proposal_id,
            depositor,
        }: QueryDepositRequest,
    ) -> Self {
        Self {
            proposal_id,
            depositor: depositor.to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Query)]
#[query(
    url = "/cosmos.gov.v1beta1.Query/Deposits",
    raw = "inner::QueryDepositsRequest"
)]
pub struct QueryDepositsRequest {
    pub proposal_id: u64,
    pub pagination: Option<PaginationRequest>,
}

impl TryFrom<inner::QueryDepositsRequest> for QueryDepositsRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryDepositsRequest {
            proposal_id,
            pagination,
        }: inner::QueryDepositsRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proposal_id,
            pagination: pagination.map(|e| e.into()),
        })
    }
}

impl From<QueryDepositsRequest> for inner::QueryDepositsRequest {
    fn from(
        QueryDepositsRequest {
            proposal_id,
            pagination: _,
        }: QueryDepositsRequest,
    ) -> Self {
        Self {
            proposal_id,
            pagination: None,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Query)]
#[query(
    url = "/cosmos.gov.v1beta1.Query/Tally",
    raw = "inner::QueryTallyResultRequest"
)]
pub struct QueryTallyResultRequest {
    pub proposal_id: u64,
}

impl TryFrom<inner::QueryTallyResultRequest> for QueryTallyResultRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryTallyResultRequest { proposal_id }: inner::QueryTallyResultRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self { proposal_id })
    }
}

impl From<QueryTallyResultRequest> for inner::QueryTallyResultRequest {
    fn from(QueryTallyResultRequest { proposal_id }: QueryTallyResultRequest) -> Self {
        Self { proposal_id }
    }
}

#[derive(Clone, PartialEq, Debug, Query)]
#[query(
    url = "/cosmos.gov.v1beta1.Query/Params",
    raw = "inner::QueryAllParamsRequest"
)]
pub struct QueryAllParamsRequest;

impl TryFrom<inner::QueryAllParamsRequest> for QueryAllParamsRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryAllParamsRequest {}: inner::QueryAllParamsRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl From<QueryAllParamsRequest> for inner::QueryAllParamsRequest {
    fn from(_: QueryAllParamsRequest) -> Self {
        Self {}
    }
}

#[derive(Clone, PartialEq, Debug, Query)]
#[query(
    url = "/cosmos.gov.v1beta1.Query/Proposer",
    raw = "inner::QueryProposerRequest"
)]
pub struct QueryProposerRequest {
    pub proposal_id: u64,
}

impl TryFrom<inner::QueryProposerRequest> for QueryProposerRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryProposerRequest { proposal_id }: inner::QueryProposerRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self { proposal_id })
    }
}

impl From<QueryProposerRequest> for inner::QueryProposerRequest {
    fn from(QueryProposerRequest { proposal_id }: QueryProposerRequest) -> Self {
        Self { proposal_id }
    }
}
