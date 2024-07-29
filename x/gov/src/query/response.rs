use gears::{
    core::errors::CoreError,
    derive::{Protobuf, Query},
    tendermint::types::proto::Protobuf,
    types::{address::AccAddress, pagination::response::PaginationResponse},
};
use serde::{Deserialize, Serialize};

use crate::{
    msg::{deposit::Deposit, weighted_vote::MsgVoteWeighted},
    params::{DepositParams, TallyParams, VotingParams},
    types::proposal::{Proposal, TallyResult},
};

mod inner {
    pub use ibc_proto::cosmos::gov::v1beta1::{
        QueryDepositResponse, QueryDepositsResponse, QueryParamsResponse, QueryProposalResponse,
        QueryProposalsResponse, QueryTallyResultResponse, QueryVoteResponse, QueryVotesResponse,
    };

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct QueryProposerResponse {
        #[prost(string, tag = "1")]
        pub proposer: String,
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query, Protobuf)]
#[proto(raw = "inner::QueryProposalResponse")]
pub struct QueryProposalResponse {
    pub proposal: Option<Proposal>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query)]
// #[query(raw = "inner::QueryProposalsResponse")]
pub struct QueryProposalsResponse {
    pub proposals: Vec<Proposal>,
    pub pagination: Option<PaginationResponse>,
}

impl TryFrom<inner::QueryProposalsResponse> for QueryProposalsResponse {
    type Error = CoreError;

    fn try_from(
        inner::QueryProposalsResponse {
            proposals,
            pagination,
        }: inner::QueryProposalsResponse,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proposals: {
                let mut result = Vec::with_capacity(proposals.len());

                for proposal in proposals {
                    result.push(proposal.try_into()?)
                }

                result
            },
            pagination: pagination.map(|e| e.into()),
        })
    }
}

impl From<QueryProposalsResponse> for inner::QueryProposalsResponse {
    fn from(
        QueryProposalsResponse {
            proposals,
            pagination,
        }: QueryProposalsResponse,
    ) -> Self {
        Self {
            proposals: proposals.into_iter().map(|this| this.into()).collect(),
            pagination: pagination.map(|e| e.into()),
        }
    }
}

impl Protobuf<inner::QueryProposalsResponse> for QueryProposalsResponse {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query, Protobuf)]
#[proto(raw = "inner::QueryVoteResponse")]
pub struct QueryVoteResponse {
    pub vote: Option<MsgVoteWeighted>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query)]
// #[proto(raw = "inner::QueryVotesResponse")]
pub struct QueryVotesResponse {
    pub votes: Vec<MsgVoteWeighted>,
    pub pagination: Option<PaginationResponse>,
}

impl TryFrom<inner::QueryVotesResponse> for QueryVotesResponse {
    type Error = CoreError;

    fn try_from(
        inner::QueryVotesResponse { votes, pagination }: inner::QueryVotesResponse,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            votes: {
                let mut result = Vec::with_capacity(votes.len());

                for vote in votes {
                    result.push(vote.try_into()?);
                }

                result
            },
            pagination: pagination.map(|e| e.into()),
        })
    }
}

impl From<QueryVotesResponse> for inner::QueryVotesResponse {
    fn from(QueryVotesResponse { votes, pagination }: QueryVotesResponse) -> Self {
        Self {
            votes: votes.into_iter().map(|e| e.into()).collect(),
            pagination: pagination.map(|e| e.into()),
        }
    }
}

impl Protobuf<inner::QueryVotesResponse> for QueryVotesResponse {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query, Protobuf)]
#[proto(raw = "inner::QueryParamsResponse")]
pub struct QueryParamsResponse {
    pub voting_params: Option<VotingParams>,
    pub deposit_params: Option<DepositParams>,
    pub tally_params: Option<TallyParams>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query, Protobuf)]
#[proto(raw = "inner::QueryDepositResponse")]
pub struct QueryDepositResponse {
    pub deposit: Option<Deposit>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query)]
// #[proto(raw = "inner::QueryDepositsResponse")]
pub struct QueryDepositsResponse {
    pub deposits: Vec<Deposit>,
    pub pagination: Option<PaginationResponse>,
}

impl TryFrom<inner::QueryDepositsResponse> for QueryDepositsResponse {
    type Error = CoreError;

    fn try_from(
        inner::QueryDepositsResponse {
            deposits,
            pagination,
        }: inner::QueryDepositsResponse,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            deposits: {
                let mut result = Vec::with_capacity(deposits.len());

                for coin in deposits {
                    result.push(
                        coin.try_into()
                            .map_err(|e| CoreError::Coins(format!("Deposit: {e}")))?,
                    );
                }

                result
            },
            pagination: pagination.map(|e| e.into()),
        })
    }
}

impl From<QueryDepositsResponse> for inner::QueryDepositsResponse {
    fn from(
        QueryDepositsResponse {
            deposits,
            pagination,
        }: QueryDepositsResponse,
    ) -> Self {
        Self {
            deposits: deposits.into_iter().map(|this| this.into()).collect(),
            pagination: pagination.map(|e| e.into()),
        }
    }
}

impl Protobuf<inner::QueryDepositsResponse> for QueryDepositsResponse {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query, Protobuf)]
#[proto(raw = "inner::QueryTallyResultResponse")]
pub struct QueryTallyResultResponse {
    pub tally: Option<TallyResult>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query)]
// #[proto(raw = "inner::QueryParamsResponse")] // TODO Protobuf
pub struct QueryAllParamsResponse {
    pub voting_params: VotingParams,
    pub deposit_params: DepositParams,
    pub tally_params: TallyParams,
}

impl TryFrom<inner::QueryParamsResponse> for QueryAllParamsResponse {
    type Error = CoreError;

    fn try_from(
        inner::QueryParamsResponse {
            voting_params,
            deposit_params,
            tally_params,
        }: inner::QueryParamsResponse,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            voting_params: voting_params
                .ok_or(CoreError::MissingField(
                    "QueryAllParamsResponse: field `voting_params`".to_owned(),
                ))?
                .try_into()?,
            deposit_params: deposit_params
                .ok_or(CoreError::MissingField(
                    "QueryAllParamsResponse: field `deposit_params`".to_owned(),
                ))?
                .try_into()?,
            tally_params: tally_params
                .ok_or(CoreError::MissingField(
                    "QueryAllParamsResponse: field `tally_params`".to_owned(),
                ))?
                .try_into()?,
        })
    }
}

impl From<QueryAllParamsResponse> for inner::QueryParamsResponse {
    fn from(
        QueryAllParamsResponse {
            voting_params,
            deposit_params,
            tally_params,
        }: QueryAllParamsResponse,
    ) -> Self {
        Self {
            voting_params: Some(voting_params.into()),
            deposit_params: Some(deposit_params.into()),
            tally_params: Some(tally_params.into()),
        }
    }
}

impl Protobuf<inner::QueryParamsResponse> for QueryAllParamsResponse {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query, Protobuf)]
#[proto(raw = "inner::QueryProposerResponse")]
pub struct QueryProposerResponse {
    proposer: AccAddress,
}
