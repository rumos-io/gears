use gears::{core::errors::CoreError, derive::Query};
use serde::{Deserialize, Serialize};

use gears::types::{account::Account, address::AccAddress};

mod inner {
    pub use gears::core::query::request::account::QueryAccountRequest;
    pub use gears::core::query::response::account::QueryAccountResponse;
}

/// QueryAccountResponse is the response type for the Query/Account RPC method.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query)]
#[query(kind = "response", raw = "inner::QueryAccountResponse")]
pub struct QueryAccountResponse {
    /// account defines the account of the corresponding address.
    pub account: Option<Account>,
}

impl TryFrom<inner::QueryAccountResponse> for QueryAccountResponse {
    type Error = CoreError;

    fn try_from(raw: inner::QueryAccountResponse) -> Result<Self, Self::Error> {
        let account = raw.account.map(|a| a.try_into()).transpose()?;
        Ok(QueryAccountResponse { account })
    }
}

impl From<QueryAccountResponse> for inner::QueryAccountResponse {
    fn from(query: QueryAccountResponse) -> inner::QueryAccountResponse {
        Self {
            account: query.account.map(Into::into),
        }
    }
}

/// QueryAccountRequest is the request type for the Query/Account RPC method.
#[derive(Clone, PartialEq, Debug, Query)]
#[query(
    kind = "request",
    raw = "inner::QueryAccountRequest",
    url = "/cosmos.auth.v1beta1.Query/Account"
)]
pub struct QueryAccountRequest {
    /// address defines the address to query for.
    pub address: AccAddress,
}

impl TryFrom<inner::QueryAccountRequest> for QueryAccountRequest {
    type Error = CoreError;

    fn try_from(raw: inner::QueryAccountRequest) -> Result<Self, Self::Error> {
        let address = AccAddress::from_bech32(&raw.address)
            .map_err(|e| Self::Error::DecodeAddress(e.to_string()))?;

        Ok(QueryAccountRequest { address })
    }
}

impl From<QueryAccountRequest> for inner::QueryAccountRequest {
    fn from(query: QueryAccountRequest) -> inner::QueryAccountRequest {
        Self {
            address: query.address.to_string(),
        }
    }
}
