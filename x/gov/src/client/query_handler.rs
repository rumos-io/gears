use bytes::Bytes;
use gears::application::handlers::client::QueryHandler;
use gears::tendermint::types::proto::Protobuf;

use crate::query::{
    request::{
        QueryAllParamsRequest, QueryDepositRequest, QueryDepositsRequest, QueryParamsRequest,
        QueryProposalRequest, QueryProposalsRequest, QueryProposerRequest, QueryTallyResultRequest,
        QueryVoteRequest, QueryVotesRequest,
    },
    response::{
        QueryAllParamsResponse, QueryDepositResponse, QueryParamsResponse, QueryProposalResponse,
        QueryProposalsResponse, QueryProposerResponse, QueryTallyResultResponse, QueryVoteResponse,
        QueryVotesResponse,
    },
    GovQuery, GovQueryResponse,
};

use super::{
    cli::query::{GovQueryCli, GovQueryCliCommands},
    GovClientHandler,
};

impl QueryHandler for GovClientHandler {
    type QueryRequest = GovQuery;

    type QueryCommands = GovQueryCli;

    type QueryResponse = GovQueryResponse;

    fn prepare_query_request(
        &self,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryRequest> {
        let result = match command.command.clone() {
            GovQueryCliCommands::Deposit {
                proposal_id,
                depositor,
            } => Self::QueryRequest::Deposit(QueryDepositRequest {
                proposal_id,
                depositor,
            }),
            GovQueryCliCommands::Deposits { proposal_id } => {
                Self::QueryRequest::Deposits(QueryDepositsRequest {
                    proposal_id,
                    pagination: None,
                })
            }
            GovQueryCliCommands::Params { kind } => {
                Self::QueryRequest::Params(QueryParamsRequest { kind })
            }
            GovQueryCliCommands::AllParams => Self::QueryRequest::AllParams(QueryAllParamsRequest),
            GovQueryCliCommands::Proposal { proposal_id } => {
                Self::QueryRequest::Proposal(QueryProposalRequest { proposal_id })
            }
            GovQueryCliCommands::Proposals {
                voter,
                depositor,
                status,
            } => Self::QueryRequest::Proposals(QueryProposalsRequest {
                voter,
                depositor,
                proposal_status: status,
                pagination: None,
            }),
            GovQueryCliCommands::Tally { proposal_id } => {
                Self::QueryRequest::Tally(QueryTallyResultRequest { proposal_id })
            }
            GovQueryCliCommands::Vote { proposal_id, voter } => {
                Self::QueryRequest::Vote(QueryVoteRequest { proposal_id, voter })
            }
            GovQueryCliCommands::Votes { proposal_id } => {
                Self::QueryRequest::Votes(QueryVotesRequest {
                    proposal_id,
                    pagination: None,
                })
            }
            GovQueryCliCommands::Proposer { proposal_id } => {
                Self::QueryRequest::Proposer(QueryProposerRequest { proposal_id })
            }
        };

        Ok(result)
    }

    fn handle_raw_response(
        &self,
        query_bytes: Vec<u8>,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryResponse> {
        let result = match &command.command {
            GovQueryCliCommands::Deposit {
                proposal_id: _,
                depositor: _,
            } => Self::QueryResponse::Deposit(QueryDepositResponse::decode::<Bytes>(
                query_bytes.into(),
            )?),
            GovQueryCliCommands::Deposits { proposal_id: _ } => Self::QueryResponse::Deposit(
                QueryDepositResponse::decode::<Bytes>(query_bytes.into())?,
            ),
            GovQueryCliCommands::Params { kind: _ } => Self::QueryResponse::Params(
                QueryParamsResponse::decode::<Bytes>(query_bytes.into())?,
            ),
            GovQueryCliCommands::AllParams => Self::QueryResponse::AllParams(
                QueryAllParamsResponse::decode::<Bytes>(query_bytes.into())?,
            ),
            GovQueryCliCommands::Proposal { proposal_id: _ } => Self::QueryResponse::Proposal(
                QueryProposalResponse::decode::<Bytes>(query_bytes.into())?,
            ),
            GovQueryCliCommands::Proposals {
                voter: _,
                depositor: _,
                status: _,
            } => Self::QueryResponse::Proposals(QueryProposalsResponse::decode::<Bytes>(
                query_bytes.into(),
            )?),
            GovQueryCliCommands::Tally { proposal_id: _ } => Self::QueryResponse::Tally(
                QueryTallyResultResponse::decode::<Bytes>(query_bytes.into())?,
            ),
            GovQueryCliCommands::Vote {
                proposal_id: _,
                voter: _,
            } => Self::QueryResponse::Vote(QueryVoteResponse::decode::<Bytes>(query_bytes.into())?),
            GovQueryCliCommands::Votes { proposal_id: _ } => {
                Self::QueryResponse::Votes(QueryVotesResponse::decode::<Bytes>(query_bytes.into())?)
            }
            GovQueryCliCommands::Proposer { proposal_id: _ } => Self::QueryResponse::Proposer(
                QueryProposerResponse::decode::<Bytes>(query_bytes.into())?,
            ),
        };

        Ok(result)
    }
}
