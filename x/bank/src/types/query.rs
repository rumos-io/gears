use gears::{
    core::errors::CoreError, derive::{Protobuf, Query}, tendermint::types::proto::Protobuf, types::{
        address::AccAddress,
        base::{coin::UnsignedCoin, errors::CoinError},
        denom::Denom,
        pagination::{request::PaginationRequest, response::PaginationResponse},
        tx::metadata::{Metadata, MetadataParseError},
    }
};
use serde::{Deserialize, Serialize};

mod inner {
    pub use gears::core::bank::Metadata;
    pub use gears::core::base::coin::Coin;
    pub use gears::core::query::request::bank::QueryAllBalancesRequest;
    pub use gears::core::query::request::bank::QueryBalanceRequest;
    pub use gears::core::query::request::bank::QueryDenomMetadataRequest;
    pub use gears::core::query::request::bank::QueryDenomsMetadataRequest;
    pub use gears::core::query::response::bank::QueryAllBalancesResponse;
    pub use gears::core::query::response::bank::QueryBalanceResponse;
    pub use gears::core::query::response::bank::QueryTotalSupplyRequest;
    pub use gears::core::query::response::bank::QueryTotalSupplyResponse;
    pub use gears::core::query::response::PageResponse;
}

#[derive(Clone, PartialEq, Debug, Query, Protobuf)]
#[query(url = "/cosmos.bank.v1beta1.Query/TotalSupply")]
#[proto(raw = "inner::QueryTotalSupplyRequest")]
pub struct QueryTotalSupplyRequest {
    pub pagination: Option<PaginationRequest>,
}
 

#[derive(Clone, PartialEq, Debug, Query, Protobuf)]
#[query(url = "/cosmos.bank.v1beta1.Query/DenomsMetadata")]
#[proto(raw = "inner::QueryDenomsMetadataRequest")]
pub struct QueryDenomsMetadataRequest {
    pub pagination: Option<PaginationRequest>,
}

/// QueryBalanceRequest is the request type for the Query/Balance RPC method.
#[derive(Clone, PartialEq, Debug, Query, Protobuf)]
#[query(
    url = "/cosmos.bank.v1beta1.Query/Balance", // TODO: are u sure?
   
)]
#[proto( raw = "inner::QueryBalanceRequest")]
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
    pub pagination: Option<PaginationRequest>,
}
 

/// QueryAllBalancesResponse is the response type for the Query/AllBalances RPC
/// method.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Query, )] //Protobuf TODO
// #[proto(raw = "inner::QueryAllBalancesResponse")]
pub struct QueryAllBalancesResponse {
    /// balances is the balances of all the coins.
    pub balances: Vec<UnsignedCoin>,
    /// pagination defines the pagination in the response.
    pub pagination: Option<PaginationResponse>,
}

impl TryFrom<inner::QueryAllBalancesResponse> for QueryAllBalancesResponse {
    type Error = CoreError;

    fn try_from(raw: inner::QueryAllBalancesResponse) -> Result<Self, Self::Error> {
        let balances = raw
            .balances
            .into_iter()
            .map(UnsignedCoin::try_from)
            .collect::<Result<Vec<UnsignedCoin>, CoinError>>()
            .map_err(|e| CoreError::Coin(e.to_string()))?;

        Ok(QueryAllBalancesResponse {
            balances,
            pagination: raw.pagination.map(PaginationResponse::from),
        })
    }
}

impl From<QueryAllBalancesResponse> for inner::QueryAllBalancesResponse {
    fn from(
        QueryAllBalancesResponse {
            balances,
            pagination,
        }: QueryAllBalancesResponse,
    ) -> inner::QueryAllBalancesResponse {
        let balances = balances.into_iter().map(inner::Coin::from).collect();

        Self {
            balances,
            pagination: pagination.map(PaginationResponse::into),
        }
    }
}

impl Protobuf<inner::QueryAllBalancesResponse> for QueryAllBalancesResponse{}

/// QueryBalanceResponse is the response type for the Query/Balance RPC method.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Query)]
// #[query(raw = "inner::QueryBalanceResponse")]
pub struct QueryBalanceResponse {
    /// balance is the balance of the coin.
    pub balance: Option<UnsignedCoin>,
}

impl TryFrom<inner::QueryBalanceResponse> for QueryBalanceResponse {
    type Error = CoreError;

    fn try_from(raw: inner::QueryBalanceResponse) -> Result<Self, Self::Error> {
        let balance = raw
            .balance
            .map(|coin| coin.try_into())
            .transpose()
            .map_err(|e: CoinError| CoreError::Coin(e.to_string()))?;
        Ok(QueryBalanceResponse { balance })
    }
}

impl From<QueryBalanceResponse> for inner::QueryBalanceResponse {
    fn from(query: QueryBalanceResponse) -> inner::QueryBalanceResponse {
        let balance = query.balance.map(|coin| coin.into());
        Self { balance }
    }
}

impl Protobuf<inner::QueryBalanceResponse> for QueryBalanceResponse{}

/// QueryTotalSupplyResponse is the response type for the Query/TotalSupply RPC
/// method
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query)]
// #[query(raw = "inner::QueryTotalSupplyResponse")]
pub struct QueryTotalSupplyResponse {
    /// supply is the supply of the coins
    pub supply: Vec<UnsignedCoin>,
    /// pagination defines the pagination in the response.
    ///
    /// Since: cosmos-sdk 0.43
    pub pagination: Option<PaginationResponse>,
}

impl TryFrom<inner::QueryTotalSupplyResponse> for QueryTotalSupplyResponse {
    type Error = CoreError;

    fn try_from(raw: inner::QueryTotalSupplyResponse) -> Result<Self, Self::Error> {
        let supply = raw
            .supply
            .into_iter()
            .map(UnsignedCoin::try_from)
            .collect::<Result<Vec<UnsignedCoin>, CoinError>>()
            .map_err(|e| CoreError::Coin(e.to_string()))?;

        Ok(Self {
            supply,
            pagination: raw.pagination.map(|this| this.into()),
        })
    }
}

impl From<QueryTotalSupplyResponse> for inner::QueryTotalSupplyResponse {
    fn from(query: QueryTotalSupplyResponse) -> inner::QueryTotalSupplyResponse {
        let supply: Vec<UnsignedCoin> = query.supply;
        let supply = supply.into_iter().map(inner::Coin::from).collect();

        Self {
            supply,
            pagination: query.pagination.map(|this| this.into()),
        }
    }
}

impl Protobuf<inner::QueryTotalSupplyResponse> for QueryTotalSupplyResponse{}

/// We use our own version of the DenomsMetadataResponse struct because the
/// Metadata struct in ibc_proto has additional fields that were added in SDK
/// v46 (uri and uri_hash).
#[derive(Clone, PartialEq, prost::Message)]
pub struct RawQueryDenomsMetadataResponse {
    /// metadata provides the client information for all the registered tokens.
    #[prost(message, repeated, tag = "1")]
    pub metadatas: ::prost::alloc::vec::Vec<inner::Metadata>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<inner::PageResponse>,
}

/// QueryDenomsMetadataResponse is the response type for the
/// Query/DenomsMetadata RPC method.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query)]
// #[query(raw = "RawQueryDenomsMetadataResponse")]
pub struct QueryDenomsMetadataResponse {
    // metadata provides the client information for all the registered tokens.
    pub metadatas: Vec<Metadata>,
    // pagination defines the pagination in the response.
    pub pagination: Option<PaginationResponse>,
}

impl TryFrom<RawQueryDenomsMetadataResponse> for QueryDenomsMetadataResponse {
    type Error = CoreError;

    fn try_from(raw: RawQueryDenomsMetadataResponse) -> Result<Self, Self::Error> {
        let metadatas = raw
            .metadatas
            .into_iter()
            .map(Metadata::try_from)
            .collect::<Result<Vec<Metadata>, MetadataParseError>>()
            .map_err(|e| CoreError::Custom(e.to_string()));

        Ok(QueryDenomsMetadataResponse {
            metadatas: metadatas?,
            pagination: raw.pagination.map(|this| this.into()),
        })
    }
}

impl From<QueryDenomsMetadataResponse> for RawQueryDenomsMetadataResponse {
    fn from(query: QueryDenomsMetadataResponse) -> RawQueryDenomsMetadataResponse {
        RawQueryDenomsMetadataResponse {
            metadatas: query
                .metadatas
                .into_iter()
                .map(inner::Metadata::from)
                .collect(),
            pagination: query.pagination.map(|this| this.into()),
        }
    }
}

impl Protobuf<RawQueryDenomsMetadataResponse> for QueryDenomsMetadataResponse{}

#[derive(Clone, Debug, PartialEq, Query, Protobuf)]
#[query(
    url = "/cosmos.bank.v1beta1.Query/DenomsMetadata",
   
)]
#[proto( raw = "inner::QueryDenomMetadataRequest")]
pub struct QueryDenomMetadataRequest {
    /// denom is the coin denom to query metadata for.
    pub denom: Denom,
}

// impl 

/// We use our own version of the QueryDenomMetadataResponse struct because the
/// Metadata struct in ibc_proto has additional fields that were added in SDK
/// v46 (uri and uri_hash).
#[derive(Clone, PartialEq, prost::Message)]
pub struct RawQueryDenomMetadataResponse {
    /// metadata describes and provides all the client information for the requested token.
    #[prost(message, optional, tag = "1")]
    pub metadata: Option<inner::Metadata>,
}

#[derive(Clone, Debug, Serialize, Query, Protobuf)]
#[proto(raw = "RawQueryDenomMetadataResponse")]
pub struct QueryDenomMetadataResponse {
    /// metadata describes and provides all the client information for the requested token.
    pub metadata: Option<Metadata>,
}

