use anyhow::{anyhow, Result};
use clap::{Arg, ArgMatches, Command};

use ibc_proto::protobuf::Protobuf;
use proto_messages::cosmos::bank::v1beta1::{QueryAllBalancesRequest, QueryAllBalancesResponse};
use proto_types::AccAddress;
use tendermint_rpc::{Client, HttpClient};
use tokio::runtime::Runtime;

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

pub fn run_bank_query_command(matches: &ArgMatches, node: &str) -> Result<String> {
    let client = HttpClient::new(node)?;

    match matches.subcommand() {
        Some(("balances", sub_matches)) => {
            let address = sub_matches
                .get_one::<AccAddress>("address")
                .expect("address argument is required preventing `None`")
                .to_owned();

            Runtime::new()
                .expect("unclear why this would ever fail")
                .block_on(get_balances(client, address))
        }
        _ => unreachable!("exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}

pub async fn get_balances(client: HttpClient, address: AccAddress) -> Result<String> {
    let query = QueryAllBalancesRequest {
        address,
        pagination: None,
    };
    let res = client
        .abci_query(
            Some(
                "/cosmos.bank.v1beta1.Query/AllBalances"
                    .parse()
                    .expect("hard coded path will always succeed"),
            ),
            query.encode_vec(),
            None,
            false,
        )
        .await?;

    if res.code.is_err() {
        return Err(anyhow!("node returned an error: {}", res.log));
    }

    let res = QueryAllBalancesResponse::decode(&*res.value)?;
    Ok(serde_json::to_string_pretty(&res)?)
}
