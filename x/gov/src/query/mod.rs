use gears::{baseapp::QueryRequest, derive::Query};
use request::{
    QueryAllParamsRequest, QueryDepositRequest, QueryDepositsRequest, QueryParamsRequest,
    QueryProposalRequest, QueryProposalsRequest, QueryProposerRequest, QueryTallyResultRequest,
    QueryVoteRequest, QueryVotesRequest,
};
use response::{
    QueryAllParamsResponse, QueryDepositResponse, QueryDepositsResponse, QueryParamsResponse,
    QueryProposalResponse, QueryProposalsResponse, QueryProposerResponse, QueryTallyResultResponse,
    QueryVoteResponse, QueryVotesResponse,
};
use serde::{Deserialize, Serialize};

use crate::proposal::Proposal;

pub mod request;
pub mod response;

#[derive(Debug, Clone, Query)]
#[query(request)]
pub enum GovQuery {
    Deposit(QueryDepositRequest),
    Deposits(QueryDepositsRequest),
    Params(QueryParamsRequest),
    AllParams(QueryAllParamsRequest),
    Proposal(QueryProposalRequest),
    Proposals(QueryProposalsRequest),
    Tally(QueryTallyResultRequest),
    Vote(QueryVoteRequest),
    Votes(QueryVotesRequest),
    Proposer(QueryProposerRequest),
}

impl QueryRequest for GovQuery {
    fn height(&self) -> u32 {
        todo!()
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query)]
#[serde(untagged)]
pub enum GovQueryResponse<T: Proposal> {
    Deposit(QueryDepositResponse),
    Deposits(QueryDepositsResponse),
    Params(QueryParamsResponse),
    AllParams(QueryAllParamsResponse),
    Proposal(QueryProposalResponse<T>),
    Proposals(QueryProposalsResponse<T>),
    Tally(QueryTallyResultResponse),
    Vote(QueryVoteResponse),
    Votes(QueryVotesResponse),
    Proposer(QueryProposerResponse),
}
