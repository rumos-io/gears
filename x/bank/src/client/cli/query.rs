use anyhow::Result;
use clap::{Arg, ArgMatches, Command};

use gears::client::query::run_query;
use ibc_proto::cosmos::bank::v1beta1::QueryAllBalancesResponse as RawQueryAllBalancesResponse;
use ibc_proto::protobuf::Protobuf;
use proto_messages::cosmos::bank::v1beta1::{QueryAllBalancesRequest, QueryAllBalancesResponse};
use proto_types::AccAddress;
use tendermint_informal::block::Height;

pub fn get_bank_query_command() -> Command {
    Command::new("bank")
        .about("Querying commands for the bank module")
        .subcommand(
            Command::new("balances")
                .about("Query for account balances by address")
                .arg(
                    Arg::new("address")
                        .required(true)
                        .value_parser(clap::value_parser!(AccAddress)),
                ),
        )
        .subcommand_required(true)
}

pub fn run_bank_query_command(
    matches: &ArgMatches,
    node: &str,
    height: Option<Height>,
) -> Result<String> {
    match matches.subcommand() {
        Some(("balances", sub_matches)) => {
            let address = sub_matches
                .get_one::<AccAddress>("address")
                .expect("address argument is required preventing `None`")
                .to_owned();

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
        _ => unreachable!("exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}
