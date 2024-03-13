use std::borrow::Cow;

use anyhow::Result;
use clap::{Args, Subcommand};

use gears::{application::handlers_v2::QueryHandler, client::query::run_query};

use serde::{Deserialize, Serialize};
use tendermint::informal::block::Height;

use prost::encoding::DecodeContext;
use prost::encoding::WireType;
use proto_messages::{
    cosmos::{
        auth::v1beta1::{QueryAccountRequest, QueryAccountResponse},
        ibc::{auth::RawQueryAccountResponse, protobuf::Protobuf},
        query::Query,
    },
    Error,
};
use proto_types::AccAddress;

#[derive(Args, Debug)]
pub struct AuthQueryCli {
    #[command(subcommand)]
    command: AuthCommands,
}

#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    /// Query for account by address
    Account {
        /// address
        address: AccAddress,
    },
}

#[derive(Clone, PartialEq)]
pub enum AuthQuery {
    Account(QueryAccountRequest),
}

/// TODO: Proc macros?
impl prost::Message for RawAuthQueryResponse {
    fn encode_raw<B>(&self, buf: &mut B)
    where
        B: bytes::BufMut,
        Self: Sized,
    {
        match self {
            RawAuthQueryResponse::Account(var) => var.encode_raw(buf),
        }
    }

    fn merge_field<B>(
        &mut self,
        tag: u32,
        wire_type: WireType,
        buf: &mut B,
        ctx: DecodeContext,
    ) -> std::prelude::v1::Result<(), prost::DecodeError>
    where
        B: bytes::Buf,
        Self: Sized,
    {
        match self {
            RawAuthQueryResponse::Account(var) => var.merge_field(tag, wire_type, buf, ctx),
        }
    }

    fn encoded_len(&self) -> usize {
        match self {
            RawAuthQueryResponse::Account(var) => var.encoded_len(),
        }
    }

    fn clear(&mut self) {
        match self {
            RawAuthQueryResponse::Account(var) => var.clear(),
        }
    }
}

impl Protobuf<RawAuthQueryResponse> for AuthQueryResponse {}

impl Query for AuthQuery {
    fn query_url(&self) -> Cow<'static, str> {
        match self {
            AuthQuery::Account(_) => Cow::Borrowed("/cosmos.auth.v1beta1.Query/Account"),
        }
    }

    fn as_bytes(self) -> Vec<u8> {
        match self {
            AuthQuery::Account(cmd) => cmd.encode_vec(),
        }
    }
}

impl TryFrom<RawAuthQueryResponse> for AuthQueryResponse {
    type Error = Error;

    fn try_from(value: RawAuthQueryResponse) -> std::prelude::v1::Result<Self, Self::Error> {
        let val = match value {
            RawAuthQueryResponse::Account(var) => Self::Account(var.try_into()?),
        };

        Ok(val)
    }
}

impl From<AuthQueryResponse> for RawAuthQueryResponse {
    fn from(value: AuthQueryResponse) -> Self {
        match value {
            AuthQueryResponse::Account(var) => Self::Account(var.into()),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum RawAuthQueryResponse {
    Account(RawQueryAccountResponse),
}

impl Default for RawAuthQueryResponse {
    fn default() -> Self {
        Self::Account(Default::default())
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum AuthQueryResponse {
    Account(QueryAccountResponse),
}

#[derive(Debug, Clone)]
pub struct AuthQueryHandler;

impl QueryHandler for AuthQueryHandler {
    type Query = AuthQuery;

    type RawQueryResponse = RawAuthQueryResponse;

    type QueryResponse = AuthQueryResponse;

    type QueryCommand = AuthQueryCli;

    fn prepare_query(
        &self,
        command: Self::QueryCommand,
        _node: &str,
        _height: Option<Height>,
    ) -> anyhow::Result<Self::Query> {
        let res = match command.command {
            AuthCommands::Account { address } => {
                AuthQuery::Account(QueryAccountRequest { address })
            }
        };

        Ok(res)
    }
}

pub fn run_auth_query_command(
    args: AuthQueryCli,
    node: &str,
    height: Option<Height>,
) -> Result<String> {
    match args.command {
        AuthCommands::Account { address } => {
            let query = QueryAccountRequest { address };

            let res = run_query::<QueryAccountResponse, RawQueryAccountResponse>(
                query.encode_vec(),
                "/cosmos.auth.v1beta1.Query/Account".into(),
                node,
                height,
            )?;

            Ok(serde_json::to_string_pretty(&res)?)
        }
    }
}
