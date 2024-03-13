use std::borrow::Cow;

use anyhow::Result;
use clap::{Args, Subcommand};

use gears::{application::handlers_v2::QueryHandler, client::query::run_query};
use prost::encoding::{DecodeContext, WireType};
use prost::Message;
use proto_messages::cosmos::{
    bank::v1beta1::{
        QueryAllBalancesRequest, QueryAllBalancesResponse, QueryDenomsMetadataRequest,
        QueryDenomsMetadataResponse, RawQueryDenomsMetadataResponse,
    },
    ibc::{bank::RawQueryAllBalancesResponse, protobuf::Protobuf},
    query::Query,
};
use proto_types::AccAddress;
use serde::Serialize;
use tendermint::informal::block::Height;

#[derive(Args, Debug)]
pub struct BankQueryCli {
    #[command(subcommand)]
    command: BankCommands,
}

#[derive(Subcommand, Debug)]
pub enum BankCommands {
    /// Query for account balances by address
    Balances {
        /// address
        address: AccAddress,
    },
    DenomMetadata,
}

pub fn run_bank_query_command(
    args: BankQueryCli,
    node: &str,
    height: Option<Height>,
) -> Result<String> {
    match args.command {
        BankCommands::Balances { address } => {
            let query = QueryAllBalancesRequest {
                address,
                pagination: None,
            };

            let res = run_query::<QueryAllBalancesResponse, RawQueryAllBalancesResponse>(
                query.encode_vec(),
                "/cosmos.bank.v1beta1.Query/AllBalances".into(),
                node,
                height,
            )?;

            Ok(serde_json::to_string_pretty(&res)?)
        }
        BankCommands::DenomMetadata => {
            let query = QueryDenomsMetadataRequest { pagination: None };

            let res = run_query::<QueryDenomsMetadataResponse, RawQueryDenomsMetadataResponse>(
                query.encode_to_vec(),
                "/cosmos.bank.v1beta1.Query/DenomsMetadata".into(),
                node,
                height,
            )?;

            Ok(serde_json::to_string_pretty(&res)?)
        }
    }
}

#[derive(Debug, Clone)]
pub struct BankQueryHandler;

impl QueryHandler for BankQueryHandler {
    type Query = BankQuery;

    type RawQueryResponse = RawBankQueryResponse;

    type QueryResponse = BankQueryResponse;

    type QueryCommands = BankQueryCli;

    fn prepare_query(
        &self,
        command: Self::QueryCommands,
        _node: &str,
        _height: Option<Height>,
    ) -> anyhow::Result<Self::Query> {
        let res = match command.command {
            BankCommands::Balances { address } => BankQuery::Balances(QueryAllBalancesRequest {
                address,
                pagination: None,
            }),
            BankCommands::DenomMetadata => {
                BankQuery::DenomMetadata(QueryDenomsMetadataRequest { pagination: None })
            }
        };

        Ok(res)
    }
}

#[derive(Clone, PartialEq)]
pub enum BankQuery {
    Balances(QueryAllBalancesRequest),
    DenomMetadata(QueryDenomsMetadataRequest),
}

impl Query for BankQuery {
    fn query_url(&self) -> Cow<'static, str> {
        match self {
            BankQuery::Balances(_) => Cow::Borrowed("/cosmos.bank.v1beta1.Query/AllBalances"),
            BankQuery::DenomMetadata(_) => {
                Cow::Borrowed("/cosmos.bank.v1beta1.Query/DenomsMetadata")
            }
        }
    }

    fn as_bytes(self) -> Vec<u8> {
        match self {
            BankQuery::Balances(var) => var.encode_vec(),
            BankQuery::DenomMetadata(var) => var.encode_to_vec(),
        }
    }
}

#[derive(Clone, Serialize)]
pub enum BankQueryResponse {
    Balances(QueryAllBalancesResponse),
    DenomMetadata(QueryDenomsMetadataResponse),
}

#[derive(Debug, Clone)]
pub enum RawBankQueryResponse {
    Balances(RawQueryAllBalancesResponse),
    DenomMetadata(RawQueryDenomsMetadataResponse),
}

impl Default for RawBankQueryResponse {
    fn default() -> Self {
        Self::Balances(Default::default())
    }
}

impl TryFrom<RawBankQueryResponse> for BankQueryResponse {
    type Error = proto_messages::Error;

    fn try_from(value: RawBankQueryResponse) -> Result<Self, Self::Error> {
        let res = match value {
            RawBankQueryResponse::Balances(var) => Self::Balances(var.try_into()?),
            RawBankQueryResponse::DenomMetadata(var) => Self::DenomMetadata(var.try_into()?),
        };

        Ok(res)
    }
}

impl From<BankQueryResponse> for RawBankQueryResponse {
    fn from(value: BankQueryResponse) -> Self {
        match value {
            BankQueryResponse::Balances(var) => Self::Balances(var.into()),
            BankQueryResponse::DenomMetadata(var) => Self::DenomMetadata(var.into()),
        }
    }
}

impl Protobuf<RawBankQueryResponse> for BankQueryResponse {}

impl prost::Message for RawBankQueryResponse {
    fn encode_raw<B>(&self, buf: &mut B)
    where
        B: bytes::BufMut,
        Self: Sized,
    {
        match self {
            RawBankQueryResponse::Balances(var) => var.encode_raw(buf),
            RawBankQueryResponse::DenomMetadata(var) => var.encode_raw(buf),
        }
    }

    fn merge_field<B>(
        &mut self,
        tag: u32,
        wire_type: WireType,
        buf: &mut B,
        ctx: DecodeContext,
    ) -> Result<(), prost::DecodeError>
    where
        B: bytes::Buf,
        Self: Sized,
    {
        match self {
            RawBankQueryResponse::Balances(var) => var.merge_field(tag, wire_type, buf, ctx),
            RawBankQueryResponse::DenomMetadata(var) => var.merge_field(tag, wire_type, buf, ctx),
        }
    }

    fn encoded_len(&self) -> usize {
        match self {
            RawBankQueryResponse::Balances(var) => var.encoded_len(),
            RawBankQueryResponse::DenomMetadata(var) => var.encoded_len(),
        }
    }

    fn clear(&mut self) {
        match self {
            RawBankQueryResponse::Balances(var) => var.clear(),
            RawBankQueryResponse::DenomMetadata(var) => var.clear(),
        }
    }
}
