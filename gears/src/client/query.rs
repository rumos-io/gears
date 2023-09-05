use anyhow::{anyhow, Result};
use clap::{arg, value_parser, ArgAction, Command};
use ibc_proto::protobuf::Protobuf;
use prost::Message;
use serde::Serialize;
use tendermint_informal::block::Height;
use tendermint_rpc::{endpoint::abci_query::AbciQuery, Client, HttpClient};
use tokio::runtime::Runtime;

pub fn get_query_command(sub_commands: Vec<Command>) -> Command {
    let mut cli = Command::new("query")
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
            arg!(--height)
                .help("Use a specific height to query state at (this can error if the node is pruning state)")
                .default_value("0")
                .value_parser(value_parser!(Height))
                .action(ArgAction::Set)
                .global(true),
        );

    for sub_command in sub_commands {
        cli = cli.subcommand(sub_command);
    }

    cli
}

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
    <Response as ibc_proto::protobuf::erased::TryFrom<Raw>>::Error: std::fmt::Display,
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
) -> Result<AbciQuery, tendermint_rpc::Error> {
    client
        .abci_query(Some(path), query_bytes, height, false)
        .await
}
