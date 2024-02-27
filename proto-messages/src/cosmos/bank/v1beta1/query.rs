use ibc_proto::cosmos::base::query::v1beta1::PageRequest as RawPageRequest;
use ibc_proto::cosmos::base::query::v1beta1::PageResponse as RawPageResponse;
use ibc_proto::{
    cosmos::bank::v1beta1::{
        QueryAllBalancesRequest as RawQueryAllBalancesRequest,
        QueryAllBalancesResponse as RawQueryAllBalancesResponse,
        QueryBalanceRequest as RawQueryBalanceRequest,
        QueryBalanceResponse as RawQueryBalanceResponse,
        QueryDenomMetadataRequest as RawQueryDenomMetadataRequest,
        QueryTotalSupplyResponse as RawQueryTotalSupplyResponse,
    },
    cosmos::base::v1beta1::Coin as RawCoin,
    Protobuf,
};
use proto_types::AccAddress;
use serde::{Deserialize, Serialize};

use crate::{
    cosmos::{
        base::v1beta1::Coin,
        tx::v1beta1::tx_metadata::{Metadata, RawMetadata},
    },
    Error,
};

pub use ibc_proto::cosmos::bank::v1beta1::QueryDenomsMetadataRequest;

/// QueryBalanceRequest is the request type for the Query/Balance RPC method.
#[derive(Clone, PartialEq)]
pub struct QueryBalanceRequest {
    /// address is the address to query balances for.
    pub address: proto_types::AccAddress,
    /// denom is the coin denom to query balances for.
    pub denom: proto_types::Denom,
}

impl TryFrom<RawQueryBalanceRequest> for QueryBalanceRequest {
    type Error = Error;

    fn try_from(raw: RawQueryBalanceRequest) -> Result<Self, Self::Error> {
        let address = AccAddress::from_bech32(&raw.address)
            .map_err(|e| Error::DecodeAddress(e.to_string()))?;

        let denom = raw
            .denom
            .try_into()
            .map_err(|_| Error::Coin(String::from("invalid denom")))?;

        Ok(QueryBalanceRequest { address, denom })
    }
}

impl From<QueryBalanceRequest> for RawQueryBalanceRequest {
    fn from(query: QueryBalanceRequest) -> RawQueryBalanceRequest {
        RawQueryBalanceRequest {
            address: query.address.to_string(),
            denom: query.denom.to_string(),
        }
    }
}

impl Protobuf<RawQueryBalanceRequest> for QueryBalanceRequest {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageRequest {
    pub key: Vec<u8>,
    pub offset: u64,
    pub limit: u64,
    pub count_total: bool,
    pub reverse: bool,
}

impl From<RawPageRequest> for PageRequest {
    fn from(value: RawPageRequest) -> Self {
        let RawPageRequest {
            key,
            offset,
            limit,
            count_total,
            reverse,
        } = value;

        Self {
            key,
            offset,
            limit,
            count_total,
            reverse,
        }
    }
}

impl From<PageRequest> for RawPageRequest {
    fn from(value: PageRequest) -> Self {
        let PageRequest {
            key,
            offset,
            limit,
            count_total,
            reverse,
        } = value;

        Self {
            key,
            offset,
            limit,
            count_total,
            reverse,
        }
    }
}

/// QueryAllBalanceRequest is the request type for the Query/AllBalances RPC method.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryAllBalancesRequest {
    /// address is the address to query balances for.
    pub address: proto_types::AccAddress,
    /// pagination defines an optional pagination for the request.
    pub pagination: Option<PageRequest>,
}

impl TryFrom<RawQueryAllBalancesRequest> for QueryAllBalancesRequest {
    type Error = Error;

    fn try_from(raw: RawQueryAllBalancesRequest) -> Result<Self, Self::Error> {
        let address = AccAddress::from_bech32(&raw.address)
            .map_err(|e| Error::DecodeAddress(e.to_string()))?;

        Ok(QueryAllBalancesRequest {
            address,
            pagination: raw.pagination.map(|this| this.into()),
        })
    }
}

impl From<QueryAllBalancesRequest> for RawQueryAllBalancesRequest {
    fn from(query: QueryAllBalancesRequest) -> RawQueryAllBalancesRequest {
        RawQueryAllBalancesRequest {
            address: query.address.to_string(),
            pagination: query.pagination.map(|this| this.into()),
        }
    }
}

impl Protobuf<RawQueryAllBalancesRequest> for QueryAllBalancesRequest {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageResponse {
    pub next_key: Vec<u8>,
    pub total: u64,
}

impl From<RawPageResponse> for PageResponse {
    fn from(value: RawPageResponse) -> Self {
        let RawPageResponse { next_key, total } = value;

        Self { next_key, total }
    }
}

impl From<PageResponse> for RawPageResponse {
    fn from(value: PageResponse) -> Self {
        let PageResponse { next_key, total } = value;

        Self { next_key, total }
    }
}

/// QueryAllBalancesResponse is the response type for the Query/AllBalances RPC
/// method.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct QueryAllBalancesResponse {
    /// balances is the balances of all the coins.
    pub balances: Vec<Coin>,
    /// pagination defines the pagination in the response.
    pub pagination: Option<PageResponse>,
}

impl TryFrom<RawQueryAllBalancesResponse> for QueryAllBalancesResponse {
    type Error = Error;

    fn try_from(raw: RawQueryAllBalancesResponse) -> Result<Self, Self::Error> {
        let balances: Result<Vec<Coin>, Error> =
            raw.balances.into_iter().map(Coin::try_from).collect();

        Ok(QueryAllBalancesResponse {
            balances: balances?,
            pagination: raw.pagination.map(|this| this.into()),
        })
    }
}

impl From<QueryAllBalancesResponse> for RawQueryAllBalancesResponse {
    fn from(query: QueryAllBalancesResponse) -> RawQueryAllBalancesResponse {
        let balances: Vec<Coin> = query.balances;
        let balances = balances.into_iter().map(RawCoin::from).collect();

        RawQueryAllBalancesResponse {
            balances,
            pagination: query.pagination.map(|this| this.into()),
        }
    }
}

impl Protobuf<RawQueryAllBalancesResponse> for QueryAllBalancesResponse {}

/// QueryBalanceResponse is the response type for the Query/Balance RPC method.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct QueryBalanceResponse {
    /// balance is the balance of the coin.
    pub balance: Option<Coin>,
}

impl TryFrom<RawQueryBalanceResponse> for QueryBalanceResponse {
    type Error = Error;

    fn try_from(raw: RawQueryBalanceResponse) -> Result<Self, Self::Error> {
        let balance = raw.balance.map(|coin| coin.try_into()).transpose()?;
        Ok(QueryBalanceResponse { balance })
    }
}

impl From<QueryBalanceResponse> for RawQueryBalanceResponse {
    fn from(query: QueryBalanceResponse) -> RawQueryBalanceResponse {
        let balance = query.balance.map(|coin| coin.into());
        RawQueryBalanceResponse { balance }
    }
}

impl Protobuf<RawQueryBalanceResponse> for QueryBalanceResponse {}

/// QueryTotalSupplyResponse is the response type for the Query/TotalSupply RPC
/// method
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryTotalSupplyResponse {
    /// supply is the supply of the coins
    pub supply: Vec<Coin>,
    /// pagination defines the pagination in the response.
    ///
    /// Since: cosmos-sdk 0.43
    pub pagination: Option<PageResponse>,
}

impl TryFrom<RawQueryTotalSupplyResponse> for QueryTotalSupplyResponse {
    type Error = Error;

    fn try_from(raw: RawQueryTotalSupplyResponse) -> Result<Self, Self::Error> {
        let supply: Result<Vec<Coin>, Error> = raw.supply.into_iter().map(Coin::try_from).collect();

        Ok(QueryTotalSupplyResponse {
            supply: supply?,
            pagination: raw.pagination.map(|this| this.into()),
        })
    }
}

impl From<QueryTotalSupplyResponse> for RawQueryTotalSupplyResponse {
    fn from(query: QueryTotalSupplyResponse) -> RawQueryTotalSupplyResponse {
        let supply: Vec<Coin> = query.supply;
        let supply = supply.into_iter().map(RawCoin::from).collect();

        RawQueryTotalSupplyResponse {
            supply,
            pagination: query.pagination.map(|this| this.into()),
        }
    }
}

impl Protobuf<RawQueryTotalSupplyResponse> for QueryTotalSupplyResponse {}

/// We use our own version of the DenomsMetadataResponse struct because the
/// Metadata struct in ibc_proto has additional fields that were added in SDK
/// v46 (uri and uri_hash).
#[derive(Clone, PartialEq, prost::Message)]
pub struct RawQueryDenomsMetadataResponse {
    /// metadata provides the client information for all the registered tokens.
    #[prost(message, repeated, tag = "1")]
    pub metadatas: ::prost::alloc::vec::Vec<RawMetadata>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<RawPageResponse>,
}

/// QueryDenomsMetadataResponse is the response type for the
/// Query/DenomsMetadata RPC method.
#[derive(Clone, Serialize, Deserialize)]
pub struct QueryDenomsMetadataResponse {
    // metadata provides the client information for all the registered tokens.
    pub metadatas: Vec<Metadata>,
    // pagination defines the pagination in the response.
    pub pagination: Option<PageResponse>,
}

impl TryFrom<RawQueryDenomsMetadataResponse> for QueryDenomsMetadataResponse {
    type Error = Error;

    fn try_from(raw: RawQueryDenomsMetadataResponse) -> Result<Self, Self::Error> {
        let metadatas: Result<Vec<Metadata>, Error> =
            raw.metadatas.into_iter().map(Metadata::try_from).collect();
        Ok(QueryDenomsMetadataResponse {
            metadatas: metadatas?,
            pagination: raw.pagination.map(|this| this.into()),
        })
    }
}

impl From<QueryDenomsMetadataResponse> for RawQueryDenomsMetadataResponse {
    fn from(query: QueryDenomsMetadataResponse) -> RawQueryDenomsMetadataResponse {
        RawQueryDenomsMetadataResponse {
            metadatas: query.metadatas.into_iter().map(RawMetadata::from).collect(),
            pagination: query.pagination.map(|this| this.into()),
        }
    }
}

impl Protobuf<RawQueryDenomsMetadataResponse> for QueryDenomsMetadataResponse {}

#[derive(Clone, PartialEq)]
pub struct QueryDenomMetadataRequest {
    /// denom is the coin denom to query balances for.
    pub denom: proto_types::Denom,
}

impl TryFrom<RawQueryDenomMetadataRequest> for QueryDenomMetadataRequest {
    type Error = Error;

    fn try_from(raw: RawQueryDenomMetadataRequest) -> Result<Self, Self::Error> {
        let denom = raw
            .denom
            .try_into()
            .map_err(|_| Error::Coin(String::from("invalid denom")))?;

        Ok(QueryDenomMetadataRequest { denom })
    }
}

impl From<QueryDenomMetadataRequest> for RawQueryDenomMetadataRequest {
    fn from(query: QueryDenomMetadataRequest) -> RawQueryDenomMetadataRequest {
        RawQueryDenomMetadataRequest {
            denom: query.denom.to_string(),
        }
    }
}

impl Protobuf<RawQueryDenomMetadataRequest> for QueryDenomMetadataRequest {}

/// We use our own version of the QueryDenomMetadataResponse struct because the
/// Metadata struct in ibc_proto has additional fields that were added in SDK
/// v46 (uri and uri_hash).
#[derive(Clone, PartialEq, prost::Message)]
pub struct RawQueryDenomMetadataResponse {
    /// metadata describes and provides all the client information for the requested token.
    #[prost(message, optional, tag = "1")]
    pub metadata: Option<RawMetadata>,
}

#[derive(Clone)]
pub struct QueryDenomMetadataResponse {
    /// metadata describes and provides all the client information for the requested token.
    pub metadata: Option<Metadata>,
}

impl TryFrom<RawQueryDenomMetadataResponse> for QueryDenomMetadataResponse {
    type Error = Error;

    fn try_from(raw: RawQueryDenomMetadataResponse) -> Result<Self, Self::Error> {
        let metadata = raw
            .metadata
            .map(Metadata::try_from)
            .transpose()
            .map_err(|_| Error::Coin(String::from("invalid metadata")))?;

        Ok(QueryDenomMetadataResponse { metadata })
    }
}

impl From<QueryDenomMetadataResponse> for RawQueryDenomMetadataResponse {
    fn from(query: QueryDenomMetadataResponse) -> RawQueryDenomMetadataResponse {
        RawQueryDenomMetadataResponse {
            metadata: query.metadata.map(RawMetadata::from),
        }
    }
}

impl Protobuf<RawQueryDenomMetadataResponse> for QueryDenomMetadataResponse {}
