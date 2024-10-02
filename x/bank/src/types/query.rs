use gears::{
    derive::{Protobuf, Query},
    types::{
        address::AccAddress,
        base::coin::UnsignedCoin,
        denom::Denom,
        pagination::{request::PaginationRequest, response::PaginationResponse},
        tx::metadata::Metadata,
    },
};
use prost::Message;
use serde::{Deserialize, Serialize};

use crate::BankParams;

pub mod inner {
    pub use ibc_proto::cosmos::bank::v1beta1::QueryAllBalancesRequest;
    pub use ibc_proto::cosmos::bank::v1beta1::QueryAllBalancesResponse;
    pub use ibc_proto::cosmos::bank::v1beta1::QueryBalanceRequest;
    pub use ibc_proto::cosmos::bank::v1beta1::QueryBalanceResponse;
    pub use ibc_proto::cosmos::bank::v1beta1::QueryDenomMetadataRequest;
    pub use ibc_proto::cosmos::bank::v1beta1::QueryDenomMetadataResponse;
    pub use ibc_proto::cosmos::bank::v1beta1::QueryDenomsMetadataRequest;
    pub use ibc_proto::cosmos::bank::v1beta1::QueryTotalSupplyRequest;
    pub use ibc_proto::cosmos::bank::v1beta1::QueryTotalSupplyResponse;
}

#[derive(Clone, PartialEq, Debug, Query, Protobuf)]
#[query(url = "/cosmos.bank.v1beta1.Query/TotalSupply")]
#[proto(raw = "inner::QueryTotalSupplyRequest")]
pub struct QueryTotalSupplyRequest {
    #[proto(optional)]
    pub pagination: Option<PaginationRequest>,
}

#[derive(Clone, PartialEq, Debug, Query, Protobuf)]
#[query(url = "/cosmos.bank.v1beta1.Query/DenomsMetadata")]
#[proto(raw = "inner::QueryDenomsMetadataRequest")]
pub struct QueryDenomsMetadataRequest {
    #[proto(optional)]
    pub pagination: Option<PaginationRequest>,
}

/// QueryBalanceRequest is the request type for the Query/Balance RPC method.
#[derive(Clone, PartialEq, Debug, Query, Protobuf)]
#[query(url = "/cosmos.bank.v1beta1.Query/Balance")]
#[proto(raw = "inner::QueryBalanceRequest")]
pub struct QueryBalanceRequest {
    /// address is the address to query balances for.
    pub address: AccAddress,
    /// denom is the coin denom to query balances for.
    pub denom: Denom,
}

/// QueryAllBalanceRequest is the request type for the Query/AllBalances RPC method.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Query, Protobuf)]
#[query(url = "/cosmos.bank.v1beta1.Query/AllBalances")]
#[proto(raw = "inner::QueryAllBalancesRequest")]
pub struct QueryAllBalancesRequest {
    /// address is the address to query balances for.
    pub address: AccAddress,
    /// pagination defines an optional pagination for the request.
    #[proto(optional)]
    pub pagination: Option<PaginationRequest>,
}

#[derive(Clone, Debug, PartialEq, Query, Protobuf)]
#[query(url = "/cosmos.bank.v1beta1.Query/DenomMetadata")]
#[proto(raw = "inner::QueryDenomMetadataRequest")]
pub struct QueryDenomMetadataRequest {
    /// denom is the coin denom to query metadata for.
    pub denom: Denom,
}

#[derive(Clone, PartialEq, Message, Query, Protobuf)]
#[query(url = "/cosmos.bank.v1beta1.Query/Params")]
#[proto(raw = "ibc_proto::cosmos::bank::v1beta1::QueryParamsRequest")]
pub struct QueryParamsRequest {}

/// QueryAllBalancesResponse is the response type for the Query/AllBalances RPC
/// method.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Query, Protobuf)]
#[proto(raw = "inner::QueryAllBalancesResponse")]
pub struct QueryAllBalancesResponse {
    /// balances is the balances of all the coins.
    #[proto(repeated)]
    pub balances: Vec<UnsignedCoin>,
    /// pagination defines the pagination in the response.
    #[proto(optional)]
    pub pagination: Option<PaginationResponse>,
}

/// QueryBalanceResponse is the response type for the Query/Balance RPC method.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Query, Protobuf)]
#[proto(raw = "inner::QueryBalanceResponse")]
pub struct QueryBalanceResponse {
    /// balance is the balance of the coin.
    #[proto(optional)]
    pub balance: Option<UnsignedCoin>,
}

/// QueryTotalSupplyResponse is the response type for the Query/TotalSupply RPC
/// method
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query, Protobuf)]
#[proto(raw = "inner::QueryTotalSupplyResponse")]
pub struct QueryTotalSupplyResponse {
    /// supply is the supply of the coins
    #[proto(repeated)]
    pub supply: Vec<UnsignedCoin>,
    /// pagination defines the pagination in the response.
    ///
    /// Since: cosmos-sdk 0.43
    #[proto(optional)]
    pub pagination: Option<PaginationResponse>,
}

/// QueryDenomsMetadataResponse is the response type for the
/// Query/DenomsMetadata RPC method.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query, Protobuf)]
#[proto(raw = "ibc_proto::cosmos::bank::v1beta1::QueryDenomsMetadataResponse")]
pub struct QueryDenomsMetadataResponse {
    // metadata provides the client information for all the registered tokens.
    #[proto(repeated)]
    pub metadatas: Vec<Metadata>,
    // pagination defines the pagination in the response.
    #[proto(optional)]
    pub pagination: Option<PaginationResponse>,
}

#[derive(Clone, Debug, Serialize, Query, Protobuf)]
#[proto(raw = "ibc_proto::cosmos::bank::v1beta1::QueryDenomMetadataResponse")]
pub struct QueryDenomMetadataResponse {
    /// metadata describes and provides all the client information for the requested token.
    #[proto(optional)]
    pub metadata: Option<Metadata>,
}

/// QueryParamsResponse is the response type for the Query/Params RPC method
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Query, Protobuf)]
#[proto(raw = "ibc_proto::cosmos::bank::v1beta1::QueryParamsResponse")]
pub struct QueryParamsResponse {
    #[proto(optional)]
    pub params: BankParams,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Query, Protobuf)]
#[proto(raw = "ibc_proto::cosmos::bank::v1beta1::QuerySupplyOfRequest")]
#[query(url = "/cosmos.bank.v1beta1.Query/TotalSupply")]
pub struct QuerySupplyOfRequest {
    pub denom: Denom,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Query, Protobuf)]
#[proto(raw = "ibc_proto::cosmos::bank::v1beta1::QuerySupplyOfResponse")]
pub struct QuerySupplyOfResponse {
    #[proto(optional)]
    pub amount: Option<UnsignedCoin>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Query, Protobuf)]
#[proto(raw = "ibc_proto::cosmos::bank::v1beta1::QuerySpendableBalancesRequest")]
#[query(url = "/cosmos.bank.v1beta1.Query/SpendableBalances")]
pub struct QuerySpendableBalancesRequest {
    pub address: AccAddress,
    #[proto(optional)]
    pub pagination: Option<PaginationRequest>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Query, Protobuf)]
#[proto(raw = "ibc_proto::cosmos::bank::v1beta1::QuerySpendableBalancesResponse")]
pub struct QuerySpendableBalancesResponse {
    #[proto(repeated)]
    pub balances: Vec<UnsignedCoin>,
    #[proto(optional)]
    pub pagination: Option<PaginationResponse>,
}
