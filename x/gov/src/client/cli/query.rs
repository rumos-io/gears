use std::borrow::Cow;

use clap::{Args, Subcommand};
use gears::{application::handlers::client::QueryHandler, types::query::Query};
use serde::{Deserialize, Serialize};

use super::GovClientHandler;

#[derive(Args, Debug)]
pub struct GovQueryCli {
    #[command(subcommand)]
    pub command: GovQueryCliCommands,
}

#[derive(Subcommand, Debug)]
pub enum GovQueryCliCommands {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum GovQueryResponse {}

#[derive(Clone, PartialEq)]
pub enum GovQuery {}

impl Query for GovQuery {
    fn query_url(&self) -> Cow<'static, str> {
        todo!()
    }

    fn into_bytes(self) -> Vec<u8> {
        todo!()
    }
}

impl QueryHandler for GovClientHandler {
    type QueryRequest = GovQuery;

    type QueryCommands = GovQueryCli;

    type QueryResponse = GovQueryResponse;

    fn prepare_query_request(
        &self,
        _command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryRequest> {
        todo!()
    }

    fn handle_raw_response(
        &self,
        _query_bytes: Vec<u8>,
        _command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryResponse> {
        todo!()
    }
}
