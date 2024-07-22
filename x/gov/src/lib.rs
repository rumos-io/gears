pub mod client;
use gears::{
    context::InfallibleContextMut,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
};

pub mod abci_handler;
pub mod errors;
pub mod genesis;
pub mod keeper;
pub mod msg;
pub mod params;
pub mod query;
pub mod submission;
pub mod types;

pub trait ProposalHandler<PSK: ParamsSubspaceKey, P> {
    fn handle<CTX: InfallibleContextMut<DB, SK>, DB: Database, SK: StoreKey>(
        &self,
        proposal: &P,
        ctx: &mut CTX,
    ) -> anyhow::Result<()>;

    fn check(proposal: &P) -> bool;
}

#[cfg(test)]
mod test {
    use gears::{
        baseapp::QueryRequest, core::errors::CoreError, derive::TODOQuery,
        tendermint::types::proto::Protobuf,
    };

    #[derive(Clone, ::prost::Message)]
    pub struct TestQueryRequestRaw {}

    #[derive(Clone)]
    pub struct TestQueryRequest {}

    impl ::gears::types::query::Query for TestQueryRequest {
        fn query_url(&self) -> &'static str {
            todo!()
        }

        fn into_bytes(self) -> Vec<u8> {
            todo!()
        }
    }

    impl TestQueryRequest {
        pub const QUERY_URL: &'static str = "var";
    }

    impl TryFrom<TestQueryRequestRaw> for TestQueryRequest {
        type Error = CoreError;

        fn try_from(_: TestQueryRequestRaw) -> Result<Self, Self::Error> {
            Ok(Self {})
        }
    }

    impl From<TestQueryRequest> for TestQueryRequestRaw {
        fn from(_: TestQueryRequest) -> Self {
            Self {}
        }
    }

    impl Protobuf<TestQueryRequestRaw> for TestQueryRequest {}

    #[derive(TODOQuery)]
    #[todo_query(kind = "request")]
    enum QueryEnumRequst {
        Test(TestQueryRequest),
    }
}
