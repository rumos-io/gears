use anyhow::{anyhow, Result};
use clap::{Arg, ArgMatches, Command};

use ibc_proto::protobuf::Protobuf;
use proto_messages::cosmos::auth::v1beta1::{QueryAccountRequest, QueryAccountResponse};
use proto_types::AccAddress;
use tendermint_rpc::{Client, HttpClient};
use tokio::runtime::Runtime;

pub fn get_auth_query_command() -> Command {
    Command::new("auth")
        .about("Querying commands for the auth module")
        .subcommand(
            Command::new("account")
                .about("Query for account by address")
                .arg(
                    Arg::new("address")
                        .required(true)
                        .value_parser(clap::value_parser!(AccAddress)),
                ),
        )
        .subcommand_required(true)
}

pub fn run_auth_query_command(matches: &ArgMatches, node: &str) -> Result<String> {
    let client = HttpClient::new(node)?;

    match matches.subcommand() {
        Some(("account", sub_matches)) => {
            let address = sub_matches
                .get_one::<AccAddress>("address")
                .expect("address argument is required preventing `None`")
                .to_owned();

            let res = Runtime::new()
                .expect("unclear why this would ever fail")
                .block_on(get_account(client, address))?;

            Ok(serde_json::to_string_pretty(&res)?)
        }
        _ => unreachable!("exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}

pub async fn get_account(client: HttpClient, address: AccAddress) -> Result<QueryAccountResponse> {
    let query = QueryAccountRequest { address };
    let res = client
        .abci_query(
            Some(
                "/cosmos.auth.v1beta1.Query/Account"
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

    Ok(QueryAccountResponse::decode(&*res.value)?)
}
