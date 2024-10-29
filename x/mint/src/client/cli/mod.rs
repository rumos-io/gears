use gears::{application::handlers::client::QueryHandler, core::Protobuf};
use query::{MintCommands, MintQueryCli};

use crate::types::query::{
    request::{
        MintQueryRequest, QueryAnnualProvisionsRequest, QueryInflationRequest, QueryParamsRequest,
    },
    response::{
        MintQueryResponse, QueryAnnualProvisionsResponse, QueryInflationResponse,
        QueryParamsResponse,
    },
};

pub mod query;

#[derive(Debug, Clone, Default)]
pub struct MintClientHandler;

impl QueryHandler for MintClientHandler {
    type QueryCommands = MintQueryCli;

    type QueryRequest = MintQueryRequest;

    type QueryResponse = MintQueryResponse;

    fn prepare_query_request(
        &self,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryRequest> {
        let request = match &command.command {
            MintCommands::Params => Self::QueryRequest::Params(QueryParamsRequest {}),
            MintCommands::Inflation => Self::QueryRequest::Inflation(QueryInflationRequest {}),
            MintCommands::AnnualProvisions => {
                Self::QueryRequest::AnnualProvisions(QueryAnnualProvisionsRequest {})
            }
        };

        Ok(request)
    }

    fn handle_raw_response(
        &self,
        bytes: Vec<u8>,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryResponse> {
        let response = match &command.command {
            MintCommands::Params => {
                Self::QueryResponse::Params(QueryParamsResponse::decode_vec(&bytes)?)
            }
            MintCommands::Inflation => {
                Self::QueryResponse::Inflation(QueryInflationResponse::decode_vec(&bytes)?)
            }
            MintCommands::AnnualProvisions => Self::QueryResponse::AnnualProvisions(
                QueryAnnualProvisionsResponse::decode_vec(&bytes)?,
            ),
        };

        Ok(response)
    }
}
