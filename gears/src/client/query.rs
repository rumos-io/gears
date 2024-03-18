use crate::application::handlers::QueryHandler;
use crate::runtime::runtime;
use anyhow::{anyhow, Result};
use prost::Message;
use proto_messages::cosmos::ibc::protobuf::Protobuf;
use serde::Serialize;
use tendermint::informal::block::Height;
use tendermint::rpc::{Client, HttpClient};

#[derive(Debug, Clone, derive_builder::Builder)]
pub struct QueryCommand<C> {
    pub node: url::Url,
    pub height: Option<Height>,

    pub inner: C,
}

pub fn run_query<Q, QC, QR, H>(
    QueryCommand {
        node,
        height,
        inner,
    }: QueryCommand<QC>,
    handler: &H,
) -> anyhow::Result<String>
where
    H: QueryHandler<QueryRequest = Q, QueryCommands = QC, QueryResponse = QR>,
    QR: Serialize,
{
    let query = handler.prepare_query_request(&inner)?;
    let query_bytes = handler.execute_query_request(query, node, height)?;

    let response = handler.handle_raw_response(query_bytes, &inner)?;
    let str_response = serde_json::to_string_pretty(&response)?;

    println!("{str_response}");

    Ok(str_response)
}

/// Convenience method for running queries
pub fn execute_query<
    Response: Protobuf<Raw> + std::convert::TryFrom<Raw>,
    Raw: Message + Default + std::convert::From<Response>,
>(
    path: String,
    query_bytes: Vec<u8>,
    node: &str,
    height: Option<Height>,
) -> Result<Response>
where
    <Response as TryFrom<Raw>>::Error: std::fmt::Display,
{
    let client = HttpClient::new(node)?;

    let res = runtime().block_on(client.abci_query(Some(path), query_bytes, height, false))?;

    if res.code.is_err() {
        return Err(anyhow!("node returned an error: {}", res.log));
    }

    Response::decode(&*res.value).map_err(|e| e.into())
}
