use ibc_proto::{
    cosmos::bank::v1beta1::{
        QueryAllBalancesRequest as RawQueryAllBalancesRequest,
        QueryAllBalancesResponse as RawQueryAllBalancesResponse,
        QueryBalanceRequest as RawQueryBalanceRequest,
        QueryBalanceResponse as RawQueryBalanceResponse,
    },
    cosmos::base::v1beta1::Coin as RawCoin,
    protobuf::Protobuf,
};
use proto_types::AccAddress;
use serde::{Deserialize, Serialize};

use crate::{cosmos::base::v1beta1::Coin, Error};

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

/// QueryAllBalanceRequest is the request type for the Query/AllBalances RPC method.
#[derive(Clone, PartialEq)]
pub struct QueryAllBalancesRequest {
    /// address is the address to query balances for.
    pub address: proto_types::AccAddress,
    /// pagination defines an optional pagination for the request.
    pub pagination: Option<ibc_proto::cosmos::base::query::v1beta1::PageRequest>,
}

impl TryFrom<RawQueryAllBalancesRequest> for QueryAllBalancesRequest {
    type Error = Error;

    fn try_from(raw: RawQueryAllBalancesRequest) -> Result<Self, Self::Error> {
        let address = AccAddress::from_bech32(&raw.address)
            .map_err(|e| Error::DecodeAddress(e.to_string()))?;

        Ok(QueryAllBalancesRequest {
            address,
            pagination: raw.pagination,
        })
    }
}

impl From<QueryAllBalancesRequest> for RawQueryAllBalancesRequest {
    fn from(query: QueryAllBalancesRequest) -> RawQueryAllBalancesRequest {
        RawQueryAllBalancesRequest {
            address: query.address.to_string(),
            pagination: query.pagination,
        }
    }
}

impl Protobuf<RawQueryAllBalancesRequest> for QueryAllBalancesRequest {}

/// QueryAllBalancesResponse is the response type for the Query/AllBalances RPC
/// method.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct QueryAllBalancesResponse {
    /// balances is the balances of all the coins.
    pub balances: Vec<Coin>,
    /// pagination defines the pagination in the response.
    pub pagination: ::core::option::Option<ibc_proto::cosmos::base::query::v1beta1::PageResponse>,
}

impl TryFrom<RawQueryAllBalancesResponse> for QueryAllBalancesResponse {
    type Error = Error;

    fn try_from(raw: RawQueryAllBalancesResponse) -> Result<Self, Self::Error> {
        let balances: Result<Vec<Coin>, Error> = raw
            .balances
            .into_iter()
            .map(|coin| Coin::try_from(coin))
            .collect();

        Ok(QueryAllBalancesResponse {
            balances: balances?,
            pagination: raw.pagination,
        })
    }
}

impl From<QueryAllBalancesResponse> for RawQueryAllBalancesResponse {
    fn from(query: QueryAllBalancesResponse) -> RawQueryAllBalancesResponse {
        let balances: Vec<Coin> = query.balances.into();
        let balances = balances
            .into_iter()
            .map(|coin| RawCoin::from(coin))
            .collect();

        RawQueryAllBalancesResponse {
            balances,
            pagination: query.pagination,
        }
    }
}

impl Protobuf<RawQueryAllBalancesResponse> for QueryAllBalancesResponse {}

/// QueryBalanceResponse is the response type for the Query/Balance RPC method.
#[derive(Clone, PartialEq, Debug)]
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
