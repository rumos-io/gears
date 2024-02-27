use anyhow::{anyhow, Result};
use clap::{arg, value_parser, Arg, ArgAction, ArgMatches, Command, Subcommand};
use prost::Message;
use proto_messages::cosmos::ibc_types::protobuf::Protobuf;
use serde::Serialize;
use tendermint::informal::block::Height;
use tendermint::rpc::{endpoint::abci_query::AbciQuery, Client, HttpClient};
use tokio::runtime::Runtime;

pub fn get_query_command<QuerySubcommand: Subcommand>() -> Command {
    let cli = Command::new("query")
        .about("Querying subcommands")
        .subcommand_required(true)
        .arg(
            arg!(--node)
                .help("<host>:<port> to Tendermint RPC interface for this chain")
                .default_value("http://localhost:26657")
                .action(ArgAction::Set)
                .global(true),
        )
        .arg(
            Arg::new("height")
                .long("height")
                .action(ArgAction::Set)
                .value_parser(value_parser!(Height))
                .global(true),
        );

    QuerySubcommand::augment_subcommands(cli)
}

pub fn run_query_command<QuerySubcommand: Subcommand, QueryCmdHandler>(
    matches: &ArgMatches,
    query_command_handler: QueryCmdHandler,
) -> Result<()>
where
    QueryCmdHandler: FnOnce(QuerySubcommand, &str, Option<Height>) -> Result<()>,
{
    let args = QuerySubcommand::from_arg_matches(matches)
        .expect("presumably this should work otherwise CLAP would have complained earlier");

    let node = matches
        .get_one::<String>("node")
        .expect("Node arg has a default value so this cannot be `None`.");

    let height = matches.get_one::<Height>("height");

    query_command_handler(args, node, height.cloned())
}

/// Convenience method for running queries
pub fn run_query<
    Response: Protobuf<Raw> + std::convert::TryFrom<Raw> + Serialize,
    Raw: Message + Default + std::convert::From<Response>,
>(
    query_bytes: Vec<u8>,
    path: String,
    node: &str,
    height: Option<Height>,
) -> Result<Response>
where
    <Response as TryFrom<Raw>>::Error: std::fmt::Display,
{
    let client = HttpClient::new(node)?;

    let res = Runtime::new()
        .expect("unclear why this would ever fail")
        .block_on(run_query_async(client, query_bytes, height, path))?;

    if res.code.is_err() {
        return Err(anyhow!("node returned an error: {}", res.log));
    }

    Response::decode(&*res.value).map_err(|e| e.into())
}

async fn run_query_async(
    client: HttpClient,
    query_bytes: Vec<u8>,
    height: Option<Height>,
    path: String,
) -> Result<AbciQuery, tendermint::rpc::Error> {
    client
        .abci_query(Some(path), query_bytes, height, false)
        .await
}
