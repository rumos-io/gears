use prost::Message;
use proto_messages::cosmos::ibc::protobuf::Protobuf;
use proto_messages::cosmos::query::Query;
use tendermint::{
    informal::block::Height,
    rpc::{Client, HttpClient},
};

use crate::runtime::runtime;

pub trait QueryHandler
where
    <Self::QueryResponse as TryFrom<Self::RawQueryResponse>>::Error: std::fmt::Display,
{
    type Query: Query;
    type RawQueryResponse: Message + Default + From<Self::QueryResponse>;
    type QueryResponse: Protobuf<Self::RawQueryResponse> + TryFrom<Self::RawQueryResponse>;
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
        node: &str,
        height: Option<Height>,
    ) -> anyhow::Result<Self::QueryResponse> {
        let client = HttpClient::new(node)?;

        let res = runtime().block_on(client.abci_query(
            Some(query.query_url().into_owned()),
            query.as_bytes(),
            height,
            false,
        ))?;

        if res.code.is_err() {
            return Err(anyhow::anyhow!("node returned an error: {}", res.log));
        }

        Self::QueryResponse::decode(&*res.value).map_err(|e| e.into())
    }
}
