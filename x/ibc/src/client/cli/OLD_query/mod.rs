use std::borrow::Cow;

use clap::{Args, Subcommand};
use gears::types::query::Query;
use serde::{Deserialize, Serialize};

use crate::ics02_client::client::cli::query::{ClientQuery, ClientQueryCli, ClientQueryResponse};

/// Querying commands for the ibc module
#[derive(Args, Debug)]
pub struct IbcQueryCli {
    #[command(subcommand)]
    pub command: IbcQueryCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum IbcQueryCommands {
    Client(ClientQueryCli),
}

#[derive(Clone, PartialEq)]
pub enum IbcQuery {
    Client(ClientQuery),
}

impl Query for IbcQuery {
    fn query_url(&self) -> Cow<'static, str> {
        match self {
            IbcQuery::Client(query) => query.query_url(),
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        match self {
            IbcQuery::Client(query) => query.into_bytes(),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum IbcQueryResponse {
    Client(ClientQueryResponse),
}
