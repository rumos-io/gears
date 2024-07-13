use gears::{
    core::errors::CoreError, ext::FallibleMapExt, rest::response::PaginationResponse,
    tendermint::types::proto::Protobuf, types::address::AccAddress,
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

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct QueryProposalResponse {
    pub proposal: Option<Proposal>,
}

impl TryFrom<inner::QueryProposalResponse> for QueryProposalResponse {
    type Error = CoreError;

    fn try_from(
        inner::QueryProposalResponse { proposal }: inner::QueryProposalResponse,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proposal: match proposal {
                Some(proposal) => Some(proposal.try_into()?),
                None => None,
            },
        })
    }
}

impl From<QueryProposalResponse> for inner::QueryProposalResponse {
    fn from(QueryProposalResponse { proposal }: QueryProposalResponse) -> Self {
        Self {
            proposal: proposal.map(|this| this.into()),
        }
    }
}

impl Protobuf<inner::QueryProposalResponse> for QueryProposalResponse {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
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

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct QueryVoteResponse {
    pub vote: Option<MsgVoteWeighted>,
}

impl TryFrom<inner::QueryVoteResponse> for QueryVoteResponse {
    type Error = CoreError;

    fn try_from(
        inner::QueryVoteResponse { vote }: inner::QueryVoteResponse,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            vote: match vote {
                Some(vote) => Some(vote.try_into()?),
                None => None,
            },
        })
    }
}

impl From<QueryVoteResponse> for inner::QueryVoteResponse {
    fn from(QueryVoteResponse { vote }: QueryVoteResponse) -> Self {
        Self {
            vote: vote.map(|e| e.into()),
        }
    }
}

impl Protobuf<inner::QueryVoteResponse> for QueryVoteResponse {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
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

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct QueryParamsResponse {
    pub voting_params: Option<VotingParams>,
    pub deposit_params: Option<DepositParams>,
    pub tally_params: Option<TallyParams>,
}

impl TryFrom<inner::QueryParamsResponse> for QueryParamsResponse {
    type Error = CoreError;

    fn try_from(
        inner::QueryParamsResponse {
            voting_params,
            deposit_params,
            tally_params,
        }: inner::QueryParamsResponse,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            voting_params: voting_params.try_map(|this| this.try_into())?,
            deposit_params: deposit_params.try_map(|this| this.try_into())?,
            tally_params: tally_params.try_map(|this| this.try_into())?,
        })
    }
}

impl From<QueryParamsResponse> for inner::QueryParamsResponse {
    fn from(
        QueryParamsResponse {
            voting_params,
            deposit_params,
            tally_params,
        }: QueryParamsResponse,
    ) -> Self {
        Self {
            voting_params: voting_params.map(|this| this.into()),
            deposit_params: deposit_params.map(|this| this.into()),
            tally_params: tally_params.map(|this| this.into()),
        }
    }
}

impl Protobuf<inner::QueryParamsResponse> for QueryParamsResponse {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct QueryDepositResponse {
    pub deposit: Option<Deposit>,
}

impl TryFrom<inner::QueryDepositResponse> for QueryDepositResponse {
    type Error = CoreError;

    fn try_from(
        inner::QueryDepositResponse { deposit }: inner::QueryDepositResponse,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            deposit: deposit.try_map(|this| this.try_into())?,
        })
    }
}

impl From<QueryDepositResponse> for inner::QueryDepositResponse {
    fn from(QueryDepositResponse { deposit }: QueryDepositResponse) -> Self {
        Self {
            deposit: deposit.map(|this| this.into()),
        }
    }
}

impl Protobuf<inner::QueryDepositResponse> for QueryDepositResponse {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
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

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct QueryTallyResultResponse {
    pub tally: Option<TallyResult>,
}

impl TryFrom<inner::QueryTallyResultResponse> for QueryTallyResultResponse {
    type Error = CoreError;

    fn try_from(
        inner::QueryTallyResultResponse { tally }: inner::QueryTallyResultResponse,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            tally: tally.try_map(|this| this.try_into())?,
        })
    }
}

impl From<QueryTallyResultResponse> for inner::QueryTallyResultResponse {
    fn from(QueryTallyResultResponse { tally }: QueryTallyResultResponse) -> Self {
        Self {
            tally: tally.map(|this| this.into()),
        }
    }
}

impl Protobuf<inner::QueryTallyResultResponse> for QueryTallyResultResponse {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
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

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct QueryProposerResponse {
    proposer: AccAddress,
}

impl TryFrom<inner::QueryProposerResponse> for QueryProposerResponse {
    type Error = CoreError;

    fn try_from(
        inner::QueryProposerResponse { proposer }: inner::QueryProposerResponse,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proposer: AccAddress::from_bech32(&proposer)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
        })
    }
}

impl From<QueryProposerResponse> for inner::QueryProposerResponse {
    fn from(QueryProposerResponse { proposer }: QueryProposerResponse) -> Self {
        Self {
            proposer: proposer.to_string(),
        }
    }
}

impl Protobuf<inner::QueryProposerResponse> for QueryProposerResponse {}
