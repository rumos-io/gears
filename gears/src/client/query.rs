use anyhow::{anyhow, Result};
use prost::Message;
use proto_messages::cosmos::ibc::protobuf::Protobuf;
use serde::Serialize;
use tendermint::informal::block::Height;
use tendermint::rpc::{endpoint::abci_query::AbciQuery, Client, HttpClient};

use crate::app_v2::handlers::QueryHandler;

#[derive(Debug, Clone, derive_builder::Builder)]
pub struct QueryCommand<C> {
    pub node: url::Url,
    pub height: Option<Height>,

    pub inner: C,
}

pub fn run_query_command<C, H: QueryHandler<QueryCommands = C>>(
    cmd: QueryCommand<C>,
    handler: &H,
) -> Result<()> {
    let QueryCommand {
        node,
        height,
        inner,
    } = cmd;

    handler.handle_query_command(inner, node.as_str(), height)
}

/// Convenience method for running queries
pub async fn run_query<
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

    let res = run_query_async(client, query_bytes, height, path).await?;

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
