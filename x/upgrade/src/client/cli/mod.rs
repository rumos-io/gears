use gears::{application::handlers::client::QueryHandler, core::Protobuf};
use prost::bytes::Bytes;
use query::{UpgradeQueryCli, UpgradeQueryCliCommands};

use crate::types::query::{
    QueryAppliedPlanRequest, QueryAppliedPlanResponse, QueryCurrentPlanRequest,
    QueryCurrentPlanResponse, QueryModuleVersionsRequest, QueryModuleVersionsResponse,
    UpgradeQueryRequest, UpgradeQueryResponse,
};

pub mod query;

#[derive(Debug, Clone)]
pub struct UpgradeClientHandler;

impl QueryHandler for UpgradeClientHandler {
    type QueryCommands = UpgradeQueryCli;

    type QueryRequest = UpgradeQueryRequest;

    type QueryResponse = UpgradeQueryResponse;

    fn prepare_query_request(
        &self,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryRequest> {
        let result = match &command.command {
            UpgradeQueryCliCommands::Plan => Self::QueryRequest::Plan(QueryCurrentPlanRequest {}),
            UpgradeQueryCliCommands::Applied { name } => {
                Self::QueryRequest::Applied(QueryAppliedPlanRequest {
                    name: name.to_owned(),
                })
            }
            UpgradeQueryCliCommands::ModuleVersions { name } => {
                Self::QueryRequest::ModuleVersions(QueryModuleVersionsRequest {
                    module_name: name
                        .as_ref()
                        .map(|this| this.to_owned())
                        .unwrap_or_default(),
                })
            }
        };

        Ok(result)
    }

    fn handle_raw_response(
        &self,
        query_bytes: Vec<u8>,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryResponse> {
        let bytes = Bytes::from(query_bytes);

        let result = match &command.command {
            UpgradeQueryCliCommands::Plan => {
                Self::QueryResponse::Plan(QueryCurrentPlanResponse::decode(bytes)?)
            }
            UpgradeQueryCliCommands::Applied { name: _ } => {
                Self::QueryResponse::Applied(QueryAppliedPlanResponse::decode(bytes)?)
            }
            UpgradeQueryCliCommands::ModuleVersions { name: _ } => {
                Self::QueryResponse::ModuleVersions(QueryModuleVersionsResponse::decode(bytes)?)
            }
        };

        Ok(result)
    }
}
