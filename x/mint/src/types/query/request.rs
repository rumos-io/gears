use gears::{
    baseapp::QueryRequest,
    derive::{Protobuf, Query, Raw},
};

#[derive(Debug, Clone, Raw, Query, Protobuf, serde::Serialize, serde::Deserialize)]
#[query(url = "/cosmos.mint.v1beta1.QueryParamsRequest")]
pub struct QueryParamsRequest {}

#[derive(Debug, Clone, Raw, Query, Protobuf, serde::Serialize, serde::Deserialize)]
#[query(url = "/cosmos.mint.v1beta1.QueryInflationRequest")]
pub struct QueryInflationRequest {}

#[derive(Debug, Clone, Raw, Query, Protobuf, serde::Serialize, serde::Deserialize)]
#[query(url = "/cosmos.mint.v1beta1.QueryAnnualProvisionsRequest")]
pub struct QueryAnnualProvisionsRequest {}

#[derive(Debug, Clone, Query)]
pub enum MintQueryRequest {
    Params(QueryParamsRequest),
    Inflation(QueryInflationRequest),
    AnnualProvisions(QueryAnnualProvisionsRequest),
}

impl QueryRequest for MintQueryRequest {
    fn height(&self) -> u32 {
        todo!()
    }
}
