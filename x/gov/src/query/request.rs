use gears::{rest::Pagination, types::address::AccAddress};

use crate::types::proposal::ProposalStatus;

#[derive(Clone, PartialEq, Debug)]
pub struct QueryProposalRequest {
    pub proposal_id: u64,
}

#[derive(Clone, PartialEq, Debug)]
pub struct QueryProposalsRequest {
    pub voter: AccAddress,
    pub depositor: AccAddress,
    pub proposal_status: ProposalStatus,
    pub pagination: Option<Pagination>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct QueryVoteRequest {
    pub proposal_id: u64,
    pub voter: AccAddress,
}

#[derive(Clone, PartialEq, Debug)]
pub struct QueryVotesRequest {
    pub proposal_id: u64,
    pub pagination: Option<Pagination>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct QueryParamsRequest {
    pub params_type: String, // TODO:NOW WHAT?
}

#[derive(Clone, PartialEq, Debug)]
pub struct QueryDepositRequest {
    pub proposal_id: u64,
    pub depositor: AccAddress,
}

#[derive(Clone, PartialEq, Debug)]
pub struct QueryDepositsRequest {
    pub proposal_id: u64,
    pub pagination: Option<Pagination>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct QueryTallyResultRequest {
    pub proposal_id: u64,
}
