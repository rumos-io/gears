use gears::types::decimal256::CosmosDecimalProtoString;
use gears::{
    derive::{Protobuf, Raw},
    types::decimal256::Decimal256,
};

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize, Raw, Protobuf)]
pub struct Minter {
    #[raw(kind(string), raw = String)]
    #[proto(
        from = "Decimal256::from_cosmos_proto_string",
        from_ref,
        into = "Decimal256::to_cosmos_proto_string",
        into_ref
    )]
    pub inflation: Decimal256,
    #[raw(kind(string), raw = String)]
    #[proto(
        from = "Decimal256::from_cosmos_proto_string",
        from_ref,
        into = "Decimal256::to_cosmos_proto_string",
        into_ref
    )]
    pub annual_provisions: Decimal256,
}
