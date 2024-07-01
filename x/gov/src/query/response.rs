use gears::rest::Pagination;
use serde::{Deserialize, Serialize};

use crate::{
    msg::{deposit::Deposit, vote::Vote},
    params::{DepositParams, TallyParams, VotingParams},
    types::proposal::{Proposal, TallyResult},
};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct QueryProposalResponse {
    pub proposal: Proposal,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct QueryProposalsResponse {
    pub proposal: Vec<Proposal>,
    pub pagination: Option<Pagination>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct QueryVoteResponse {
    pub vote: Vote,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct QueryVotesResponse {
    pub votes: Vec<Vote>,
    pub pagination: Option<Pagination>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct QueryParamsResponse {
    pub voting_params: VotingParams,
    pub deposit_params: DepositParams,
    pub tally_params: TallyParams,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct QueryDepositResponse {
    pub deposit: Deposit,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct QueryDepositsResponse {
    pub deposits: Vec<Deposit>,
    pub pagination: Option<Pagination>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct QueryTallyResultResponse {
    tally: TallyResult,
}
