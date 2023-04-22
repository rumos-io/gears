use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use clap::{Arg, ArgMatches, Command};

use ibc_proto::protobuf::Protobuf;
use proto_messages::cosmos::bank::v1beta1::{QueryAllBalancesRequest, QueryAllBalancesResponse};
use proto_types::AccAddress;
use tendermint_rpc::{Client, HttpClient};
use tokio::runtime::Runtime;

// TODO: use clap derive
// TODO: get clap to parse node url and address

pub fn get_bank_query_command() -> Command {
    Command::new("bank")
        .about("Querying commands for the bank module")
        .subcommand(
            Command::new("balances")
                .about("Query for account balances by address")
                .arg(Arg::new("address").required(true)),
        )
        .subcommand_required(true)
}

pub fn run_bank_query_command(matches: &ArgMatches, node: &str) -> Result<String> {
    let client = HttpClient::new(node)?;

    match matches.subcommand() {
        Some(("balances", sub_matches)) => {
            let address = sub_matches
                .get_one::<String>("address")
                .expect("address argument is required preventing `None`")
                .to_owned();

            Runtime::new()
                .expect("unclear why this would ever fail")
                .block_on(get_balances(client, address))
        }
        _ => unreachable!("exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}

pub async fn get_balances(client: HttpClient, address: String) -> Result<String> {
    let query = QueryAllBalancesRequest {
        address: AccAddress::from_str(&address).with_context(|| "invalid address")?,
        pagination: None,
    };
    let res = client
        .abci_query(
            Some("/cosmos.bank.v1beta1.Query/AllBalances".into()),
            query
                .encode_vec()
                .expect("library call will never return an error - this is a bug in the library"),
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
