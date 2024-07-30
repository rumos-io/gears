use crate::AuthsParams;
use gears::{
    core::errors::CoreError,
    derive::Query,
    types::{
        account::Account,
        address::AccAddress,
        pagination::{request::PaginationRequest, response::PaginationResponse},
    },
};
use prost::Message;
use serde::{Deserialize, Serialize};

mod inner {
    pub use gears::core::query::request::auth::QueryAccountRequest;
    pub use gears::core::query::response::auth::QueryAccountResponse;
    pub use ibc_proto::cosmos::auth::v1beta1::QueryAccountsRequest;
    pub use ibc_proto::cosmos::auth::v1beta1::QueryAccountsResponse;
    pub use ibc_proto::cosmos::auth::v1beta1::QueryParamsRequest;
    pub use ibc_proto::cosmos::auth::v1beta1::QueryParamsResponse;
}

#[derive(Clone, PartialEq, Message, Query)]
#[query(raw = "QueryParamsRequest", url = "/cosmos.auth.v1beta1.Query/Params")]
pub struct QueryParamsRequest {}

impl From<inner::QueryParamsRequest> for QueryParamsRequest {
    fn from(_value: inner::QueryParamsRequest) -> Self {
        QueryParamsRequest {}
    }
}

/// QueryAccountResponse is the response type for the Query/Account RPC method.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query)]
#[query(response, raw = "inner::QueryAccountResponse")]
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

/// QueryAccountsResponse is the response type for the Query/Accounts RPC method.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query)]
#[query(response, raw = "inner::QueryAccountsResponse")]
pub struct QueryAccountsResponse {
    /// accounts are the existing accounts
    pub accounts: Vec<Account>,
    /// pagination defines the pagination in the response.
    pub pagination: Option<PaginationResponse>,
}

impl TryFrom<inner::QueryAccountsResponse> for QueryAccountsResponse {
    type Error = CoreError;

    fn try_from(
        inner::QueryAccountsResponse {
            accounts,
            pagination,
        }: inner::QueryAccountsResponse,
    ) -> Result<Self, Self::Error> {
        let mut accounts_res = Vec::with_capacity(accounts.len());
        for raw in accounts {
            accounts_res.push(raw.try_into()?);
        }
        Ok(QueryAccountsResponse {
            accounts: accounts_res,
            pagination: pagination.map(Into::into),
        })
    }
}

impl From<QueryAccountsResponse> for inner::QueryAccountsResponse {
    fn from(
        QueryAccountsResponse {
            accounts,
            pagination,
        }: QueryAccountsResponse,
    ) -> inner::QueryAccountsResponse {
        Self {
            accounts: accounts.into_iter().map(Into::into).collect(),
            pagination: pagination.map(Into::into),
        }
    }
}

/// QueryAccountRequest is the request type for the Query/Account RPC method.
#[derive(Clone, PartialEq, Debug, Query)]
#[query(
    request,
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

/// QueryAccountsRequest is the request type for the Query/Accounts RPC method.
#[derive(Clone, PartialEq, Debug, Query)]
#[query(
    request,
    raw = "inner::QueryAccountsRequest",
    url = "/cosmos.auth.v1beta1.Query/Accounts"
)]
pub struct QueryAccountsRequest {
    /// pagination defines an optional pagination for the request.
    pub pagination: PaginationRequest,
}

impl TryFrom<inner::QueryAccountsRequest> for QueryAccountsRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryAccountsRequest { pagination }: inner::QueryAccountsRequest,
    ) -> Result<Self, Self::Error> {
        Ok(QueryAccountsRequest {
            pagination: pagination
                .ok_or(CoreError::MissingField(
                    "Missing field 'pagination'.".into(),
                ))?
                .into(),
        })
    }
}

impl From<QueryAccountsRequest> for inner::QueryAccountsRequest {
    fn from(
        QueryAccountsRequest { pagination }: QueryAccountsRequest,
    ) -> inner::QueryAccountsRequest {
        Self {
            pagination: Some(pagination.into()),
        }
    }
}

/// QueryParamsResponse is the response type for the Query/Params RPC method
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Query)]
#[query(raw = "inner::QueryParamsResponse")]
pub struct QueryParamsResponse {
    pub params: AuthsParams,
}

impl From<QueryParamsResponse> for inner::QueryParamsResponse {
    fn from(QueryParamsResponse { params }: QueryParamsResponse) -> Self {
        Self {
            params: Some(params.into()),
        }
    }
}

impl TryFrom<inner::QueryParamsResponse> for QueryParamsResponse {
    type Error = CoreError;

    fn try_from(
        inner::QueryParamsResponse { params }: inner::QueryParamsResponse,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            params: params
                .ok_or(CoreError::MissingField("Missing field 'params'.".into()))?
                .try_into()
                .map_err(|e| CoreError::DecodeGeneral(format!("{e}")))?,
        })
    }
}
