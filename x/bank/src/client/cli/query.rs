use anyhow::Result;
use clap::{Args, Subcommand};

use gears::client::query::run_query;
use prost::Message;
use proto_messages::cosmos::{
    bank::v1beta1::{
        QueryAllBalancesRequest, QueryAllBalancesResponse, QueryDenomsMetadataRequest,
        QueryDenomsMetadataResponse, RawQueryDenomsMetadataResponse,
    },
    ibc::{bank::RawQueryAllBalancesResponse, protobuf::Protobuf},
};
use proto_types::AccAddress;
use tendermint::informal::block::Height;

#[derive(Args, Debug)]
pub struct QueryCli {
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
    args: QueryCli,
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
