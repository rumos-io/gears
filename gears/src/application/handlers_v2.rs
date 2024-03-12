use prost::Message;
use proto_messages::cosmos::ibc::protobuf::Protobuf;
use proto_messages::cosmos::query::QueryUrl;
use serde::Serialize;
use tendermint::{
    informal::block::Height,
    rpc::{Client, HttpClient},
};

use crate::runtime::runtime;

pub trait QueryHandler
where
    <Self::QueryResponse as TryFrom<Self::RawQueryResponse>>::Error: std::fmt::Display,
{
    type Query: QueryUrl + Message;
    type RawQueryResponse: Message + Default + std::convert::From<Self::QueryResponse>;
    type QueryResponse: Protobuf<Self::RawQueryResponse>
        + std::convert::TryFrom<Self::RawQueryResponse>
        + Serialize;
    type QueryCommand;

    fn prepare_query(
        &self,
        command: Self::QueryCommand,
        node: &str,
        height: Option<Height>,
    ) -> anyhow::Result<Self::Query>;

    fn handle_query(
        &self,
        query: Self::Query,
        path: String,
        node: &str,
        height: Option<Height>,
    ) -> anyhow::Result<Self::QueryResponse> {
        let client = HttpClient::new(node)?;

        let res = runtime().block_on(client.abci_query(
            Some(path),
            query.encode_to_vec(),
            height,
            false,
        ))?;

        if res.code.is_err() {
            return Err(anyhow::anyhow!("node returned an error: {}", res.log));
        }

        Self::QueryResponse::decode(&*res.value).map_err(|e| e.into())
    }
}
