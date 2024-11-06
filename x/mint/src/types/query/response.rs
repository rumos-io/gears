use gears::{
    derive::{Protobuf, Query, Raw},
    types::decimal256::{CosmosDecimalProtoString, Decimal256},
};

use crate::params::{MintParams, RawMintParams};

#[derive(Debug, Clone, Raw, Query, Protobuf, serde::Serialize, serde::Deserialize)]
pub struct QueryParamsResponse {
    #[raw(kind(message), raw = RawMintParams)]
    pub params: MintParams,
}

#[derive(Debug, Clone, Raw, Query, Protobuf, serde::Serialize, serde::Deserialize)]
pub struct QueryInflationResponse {
    #[raw(kind(string), raw = String)]
    #[proto(
        from = "CosmosDecimalProtoString::from_cosmos_proto_string",
        from_ref,
        into = "CosmosDecimalProtoString::to_cosmos_proto_string",
        into_ref
    )]
    pub inflation: Decimal256,
}

#[derive(Debug, Clone, Raw, Query, Protobuf, serde::Serialize, serde::Deserialize)]
pub struct QueryAnnualProvisionsResponse {
    #[raw(kind(string), raw = String)]
    #[proto(
        from = "CosmosDecimalProtoString::from_cosmos_proto_string",
        from_ref,
        into = "CosmosDecimalProtoString::to_cosmos_proto_string",
        into_ref
    )]
    pub annual_provisions: Decimal256,
}

#[derive(Debug, Clone, Query, serde::Serialize, serde::Deserialize)]
pub enum MintQueryResponse {
    Params(QueryParamsResponse),
    Inflation(QueryInflationResponse),
    AnnualProvisions(QueryAnnualProvisionsResponse),
}
