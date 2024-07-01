use bytes::Bytes;
use gears::application::handlers::client::QueryHandler;
use gears::tendermint::types::proto::Protobuf;

use crate::query::{
    request::{
        QueryDepositRequest, QueryDepositsRequest, QueryParamsRequest, QueryProposalRequest,
        QueryProposalsRequest, QueryTallyResultRequest, QueryVoteRequest, QueryVotesRequest,
    },
    response::QueryDepositResponse,
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
        };

        Ok(result)
    }

    fn handle_raw_response(
        &self,
        _query_bytes: Vec<u8>,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryResponse> {
        let result = match &command.command {
            GovQueryCliCommands::Deposit {
                proposal_id: _,
                depositor: _,
            } => todo!(), // Self::QueryResponse::Deposit(QueryDepositResponse::decode::<Bytes>(  query_bytes.into(),)?)
            GovQueryCliCommands::Deposits { proposal_id: _ } => todo!(),
            GovQueryCliCommands::Params { kind: _ } => todo!(),
            GovQueryCliCommands::Proposal { proposal_id: _ } => todo!(),
            GovQueryCliCommands::Proposals {
                voter: _,
                depositor: _,
                status: _,
            } => todo!(),
            GovQueryCliCommands::Tally { proposal_id: _ } => todo!(),
            GovQueryCliCommands::Vote {
                proposal_id: _,
                voter: _,
            } => todo!(),
            GovQueryCliCommands::Votes { proposal_id: _ } => todo!(),
        };

        Ok(result)
    }
}
