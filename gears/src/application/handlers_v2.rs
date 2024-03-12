use proto_messages::cosmos::query::QueryUrl;
use tendermint::informal::block::Height;

pub trait QueryHandler {
    type Query: QueryUrl;
    type QueryResponse;
    type QueryCommand;

    fn prepare_query(
        &self,
        command: Self::QueryCommand,
        node: &str,
        height: Option<Height>,
    ) -> anyhow::Result<Self::Query>;

    fn handle_query(&self) -> anyhow::Result<Self::QueryResponse>; // TODO: default impl?
}
