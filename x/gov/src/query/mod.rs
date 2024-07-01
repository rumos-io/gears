use std::borrow::Cow;

use gears::{
    baseapp::{QueryRequest, QueryResponse},
    error::IBC_ENCODE_UNWRAP,
    tendermint::types::proto::Protobuf,
    types::query::Query,
};
use request::{
    QueryDepositRequest, QueryDepositsRequest, QueryParamsRequest, QueryProposalRequest,
    QueryProposalsRequest, QueryTallyResultRequest, QueryVoteRequest, QueryVotesRequest,
};
use response::{
    QueryDepositResponse, QueryDepositsResponse, QueryParamsResponse, QueryProposalResponse,
    QueryProposalsResponse, QueryTallyResultResponse, QueryVoteResponse, QueryVotesResponse,
};
use serde::{Deserialize, Serialize};

pub mod request;
pub mod response;

#[derive(Debug, Clone)]
pub enum GovQuery {
    Deposit(QueryDepositRequest),
    Deposits(QueryDepositsRequest),
    Params(QueryParamsRequest),
    Proposal(QueryProposalRequest),
    Proposals(QueryProposalsRequest),
    Tally(QueryTallyResultRequest),
    Vote(QueryVoteRequest),
    Votes(QueryVotesRequest),
}

impl Query for GovQuery {
    fn query_url(&self) -> Cow<'static, str> {
        match self {
            GovQuery::Deposit(_) => Cow::Borrowed("/cosmos.gov.v1beta1.Query/Deposit"),
            GovQuery::Deposits(_) => Cow::Borrowed("/cosmos.gov.v1beta1.Query/Deposits"),
            GovQuery::Params(_) => Cow::Borrowed("/cosmos.gov.v1beta1.Query/Params"),
            GovQuery::Proposal(_) => Cow::Borrowed("/cosmos.gov.v1beta1.Query/Proposal"),
            GovQuery::Proposals(_) => Cow::Borrowed("/cosmos.gov.v1beta1.Query/Proposals"),
            GovQuery::Tally(_) => Cow::Borrowed("/cosmos.gov.v1beta1.Query/Tally"),
            GovQuery::Vote(_) => Cow::Borrowed("/cosmos.gov.v1beta1.Query/Vote"),
            GovQuery::Votes(_) => Cow::Borrowed("/cosmos.gov.v1beta1.Query/Votes"),
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        match self {
            GovQuery::Deposit(q) => q.encode_vec().expect(IBC_ENCODE_UNWRAP),
            GovQuery::Deposits(q) => q.encode_vec().expect(IBC_ENCODE_UNWRAP),
            GovQuery::Params(q) => q.encode_vec().expect(IBC_ENCODE_UNWRAP),
            GovQuery::Proposal(q) => q.encode_vec().expect(IBC_ENCODE_UNWRAP),
            GovQuery::Proposals(q) => q.encode_vec().expect(IBC_ENCODE_UNWRAP),
            GovQuery::Tally(q) => q.encode_vec().expect(IBC_ENCODE_UNWRAP),
            GovQuery::Vote(q) => q.encode_vec().expect(IBC_ENCODE_UNWRAP),
            GovQuery::Votes(q) => q.encode_vec().expect(IBC_ENCODE_UNWRAP),
        }
    }
}

impl QueryRequest for GovQuery {
    fn height(&self) -> u32 {
        todo!()
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum GovQueryResponse {
    Deposit(QueryDepositResponse),
    Deposits(QueryDepositsResponse),
    Params(QueryParamsResponse),
    Proposal(QueryProposalResponse),
    Proposals(QueryProposalsResponse),
    Tally(QueryTallyResultResponse),
    Vote(QueryVoteResponse),
    Votes(QueryVotesResponse),
}

impl QueryResponse for GovQueryResponse {}
