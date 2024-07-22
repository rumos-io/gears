use gears::{
    core::errors::CoreError,
    derive::Query,
    types::{
        address::AccAddress,
        base::{coin::UnsignedCoin, errors::CoinError},
        denom::Denom,
        pagination::{request::PaginationRequest, response::PaginationResponse},
        tx::metadata::{Metadata, MetadataParseError},
    },
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

#[derive(Clone, PartialEq, Debug, Query)]
#[query(
    url = "/cosmos.bank.v1beta1.Query/TotalSupply",
    raw = "inner::QueryTotalSupplyRequest"
)]
pub struct QueryTotalSupplyRequest {
    pub pagination: Option<PaginationRequest>,
}

impl TryFrom<inner::QueryTotalSupplyRequest> for QueryTotalSupplyRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryTotalSupplyRequest { pagination }: inner::QueryTotalSupplyRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            pagination: pagination.map(PaginationRequest::from),
        })
    }
}

impl From<QueryTotalSupplyRequest> for inner::QueryTotalSupplyRequest {
    fn from(QueryTotalSupplyRequest { pagination }: QueryTotalSupplyRequest) -> Self {
        Self {
            pagination: pagination.map(PaginationRequest::into),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Query)]
#[query(
    url = "/cosmos.bank.v1beta1.Query/DenomsMetadata",
    raw = "inner::QueryDenomsMetadataRequest"
)]
pub struct QueryDenomsMetadataRequest {
    pub pagination: Option<PaginationRequest>,
}

impl TryFrom<inner::QueryDenomsMetadataRequest> for QueryDenomsMetadataRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryDenomsMetadataRequest { pagination }: inner::QueryDenomsMetadataRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            pagination: pagination.map(PaginationRequest::from),
        })
    }
}

impl From<QueryDenomsMetadataRequest> for inner::QueryDenomsMetadataRequest {
    fn from(QueryDenomsMetadataRequest { pagination }: QueryDenomsMetadataRequest) -> Self {
        Self {
            pagination: pagination.map(PaginationRequest::into),
        }
    }
}

/// QueryBalanceRequest is the request type for the Query/Balance RPC method.
#[derive(Clone, PartialEq, Debug, Query)]
#[query(
    url = "/cosmos.bank.v1beta1.Query/Balance", // TODO: are u sure?
    raw = "inner::QueryBalanceRequest"
)]
pub struct QueryBalanceRequest {
    /// address is the address to query balances for.
    pub address: AccAddress,
    /// denom is the coin denom to query balances for.
    pub denom: Denom,
}

impl TryFrom<inner::QueryBalanceRequest> for QueryBalanceRequest {
    type Error = CoreError;

    fn try_from(raw: inner::QueryBalanceRequest) -> Result<Self, Self::Error> {
        let address = AccAddress::from_bech32(&raw.address)
            .map_err(|e| CoreError::DecodeAddress(e.to_string()))?;

        let denom = raw
            .denom
            .try_into()
            .map_err(|_| CoreError::Coin(String::from("invalid denom")))?;

        Ok(QueryBalanceRequest { address, denom })
    }
}

impl From<QueryBalanceRequest> for inner::QueryBalanceRequest {
    fn from(query: QueryBalanceRequest) -> inner::QueryBalanceRequest {
        Self {
            address: query.address.to_string(),
            denom: query.denom.to_string(),
        }
    }
}

/// QueryAllBalanceRequest is the request type for the Query/AllBalances RPC method.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Query)]
#[query(
    url = "/cosmos.bank.v1beta1.Query/AllBalances",
    raw = "inner::QueryAllBalancesRequest"
)]
pub struct QueryAllBalancesRequest {
    /// address is the address to query balances for.
    pub address: AccAddress,
    /// pagination defines an optional pagination for the request.
    pub pagination: Option<PaginationRequest>,
}

impl TryFrom<inner::QueryAllBalancesRequest> for QueryAllBalancesRequest {
    type Error = CoreError;

    fn try_from(raw: inner::QueryAllBalancesRequest) -> Result<Self, Self::Error> {
        let address = AccAddress::from_bech32(&raw.address)
            .map_err(|e| CoreError::DecodeAddress(e.to_string()))?;

        Ok(Self {
            address,
            pagination: raw.pagination.map(PaginationRequest::from),
        })
    }
}

impl From<QueryAllBalancesRequest> for inner::QueryAllBalancesRequest {
    fn from(query: QueryAllBalancesRequest) -> inner::QueryAllBalancesRequest {
        Self {
            address: query.address.to_string(),
            pagination: query.pagination.map(PaginationRequest::into),
        }
    }
}

/// QueryAllBalancesResponse is the response type for the Query/AllBalances RPC
/// method.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Query)]
#[query(raw = "inner::QueryAllBalancesResponse")]
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

/// QueryBalanceResponse is the response type for the Query/Balance RPC method.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Query)]
#[query(raw = "inner::QueryBalanceResponse")]
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

/// QueryTotalSupplyResponse is the response type for the Query/TotalSupply RPC
/// method
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query)]
#[query(raw = "inner::QueryTotalSupplyResponse")]
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
#[query(raw = "RawQueryDenomsMetadataResponse")]
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

#[derive(Clone, PartialEq, Query)]
#[query(
    url = "/cosmos.bank.v1beta1.Query/DenomsMetadata",
    raw = "inner::QueryDenomMetadataRequest"
)]
pub struct QueryDenomMetadataRequest {
    /// denom is the coin denom to query metadata for.
    pub denom: Denom,
}

impl TryFrom<inner::QueryDenomMetadataRequest> for QueryDenomMetadataRequest {
    type Error = CoreError;

    fn try_from(raw: inner::QueryDenomMetadataRequest) -> Result<Self, Self::Error> {
        let denom = raw
            .denom
            .try_into()
            .map_err(|_| Self::Error::Coin(String::from("invalid denom")))?;

        Ok(QueryDenomMetadataRequest { denom })
    }
}

impl From<QueryDenomMetadataRequest> for inner::QueryDenomMetadataRequest {
    fn from(query: QueryDenomMetadataRequest) -> inner::QueryDenomMetadataRequest {
        Self {
            denom: query.denom.to_string(),
        }
    }
}

/// We use our own version of the QueryDenomMetadataResponse struct because the
/// Metadata struct in ibc_proto has additional fields that were added in SDK
/// v46 (uri and uri_hash).
#[derive(Clone, PartialEq, prost::Message)]
pub struct RawQueryDenomMetadataResponse {
    /// metadata describes and provides all the client information for the requested token.
    #[prost(message, optional, tag = "1")]
    pub metadata: Option<inner::Metadata>,
}

#[derive(Clone, Serialize, Query)]
#[query(raw = "RawQueryDenomMetadataResponse")]
pub struct QueryDenomMetadataResponse {
    /// metadata describes and provides all the client information for the requested token.
    pub metadata: Option<Metadata>,
}

impl TryFrom<RawQueryDenomMetadataResponse> for QueryDenomMetadataResponse {
    type Error = CoreError;

    fn try_from(raw: RawQueryDenomMetadataResponse) -> Result<Self, Self::Error> {
        let metadata = raw
            .metadata
            .map(Metadata::try_from)
            .transpose()
            .map_err(|_| CoreError::Coin(String::from("invalid metadata")))?;

        Ok(QueryDenomMetadataResponse { metadata })
    }
}

impl From<QueryDenomMetadataResponse> for RawQueryDenomMetadataResponse {
    fn from(query: QueryDenomMetadataResponse) -> RawQueryDenomMetadataResponse {
        RawQueryDenomMetadataResponse {
            metadata: query.metadata.map(inner::Metadata::from),
        }
    }
}
