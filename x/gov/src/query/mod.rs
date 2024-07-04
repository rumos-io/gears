use bytes::Bytes;
use gears::{
    baseapp::{QueryRequest, QueryResponse},
    error::IBC_ENCODE_UNWRAP,
    tendermint::types::proto::Protobuf,
    types::query::Query,
};
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

pub mod request;
pub mod response;

#[derive(Debug, Clone)]
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

impl Query for GovQuery {
    fn query_url(&self) -> &'static str {
        match self {
            GovQuery::Deposit(_) => QueryDepositRequest::QUERY_URL,
            GovQuery::Deposits(_) => QueryDepositsRequest::QUERY_URL,
            GovQuery::Params(_) => QueryParamsRequest::QUERY_URL,
            GovQuery::AllParams(_) => QueryAllParamsRequest::QUERY_URL,
            GovQuery::Proposal(_) => QueryProposalRequest::QUERY_URL,
            GovQuery::Proposals(_) => QueryProposalsRequest::QUERY_URL,
            GovQuery::Tally(_) => QueryTallyResultRequest::QUERY_URL,
            GovQuery::Vote(_) => QueryVoteRequest::QUERY_URL,
            GovQuery::Votes(_) => QueryVotesRequest::QUERY_URL,
            GovQuery::Proposer(_) => QueryProposerRequest::QUERY_URL,
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        match self {
            GovQuery::Deposit(q) => q.encode_vec().expect(IBC_ENCODE_UNWRAP),
            GovQuery::Deposits(q) => q.encode_vec().expect(IBC_ENCODE_UNWRAP),
            GovQuery::Params(q) => q.encode_vec().expect(IBC_ENCODE_UNWRAP),
            GovQuery::AllParams(q) => q.encode_vec().expect(IBC_ENCODE_UNWRAP),
            GovQuery::Proposal(q) => q.encode_vec().expect(IBC_ENCODE_UNWRAP),
            GovQuery::Proposals(q) => q.encode_vec().expect(IBC_ENCODE_UNWRAP),
            GovQuery::Tally(q) => q.encode_vec().expect(IBC_ENCODE_UNWRAP),
            GovQuery::Vote(q) => q.encode_vec().expect(IBC_ENCODE_UNWRAP),
            GovQuery::Votes(q) => q.encode_vec().expect(IBC_ENCODE_UNWRAP),
            GovQuery::Proposer(q) => q.encode_vec().expect(IBC_ENCODE_UNWRAP),
        }
    }
}

impl QueryRequest for GovQuery {
    fn height(&self) -> u32 {
        todo!()
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum GovQueryResponse {
    Deposit(QueryDepositResponse),
    Deposits(QueryDepositsResponse),
    Params(QueryParamsResponse),
    AllParams(QueryAllParamsResponse),
    Proposal(QueryProposalResponse),
    Proposals(QueryProposalsResponse),
    Tally(QueryTallyResultResponse),
    Vote(QueryVoteResponse),
    Votes(QueryVotesResponse),
    Proposer(QueryProposerResponse),
}

impl GovQueryResponse {
    pub fn encode_to_vec(&self) -> Bytes {
        match self {
            GovQueryResponse::Deposit(q) => q.encode_vec(),
            GovQueryResponse::Deposits(q) => q.encode_vec(),
            GovQueryResponse::Params(q) => q.encode_vec(),
            GovQueryResponse::AllParams(q) => q.encode_vec(),
            GovQueryResponse::Proposal(q) => q.encode_vec(),
            GovQueryResponse::Proposals(q) => q.encode_vec(),
            GovQueryResponse::Tally(q) => q.encode_vec(),
            GovQueryResponse::Vote(q) => q.encode_vec(),
            GovQueryResponse::Votes(q) => q.encode_vec(),
            GovQueryResponse::Proposer(q) => q.encode_vec(),
        }
        .expect(IBC_ENCODE_UNWRAP)
        .into()
    }
}

impl QueryResponse for GovQueryResponse {}
